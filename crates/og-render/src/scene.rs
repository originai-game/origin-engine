//! The renderer's input: everything a frame needs, with no knowledge of the ECS.
//!
//! A [`RenderScene`] owns its resources (meshes, textures, materials) and hands
//! out handles for them; instances, the camera, the light and the environment
//! are plain public fields the caller rewrites every frame. Resources are
//! append-only: a handle stays valid for the scene's lifetime; materials,
//! unskinned meshes and textures can be replaced in place
//! ([`RenderScene::set_material`], [`RenderScene::set_mesh`],
//! [`RenderScene::set_texture`]).

use crate::effects::{
    AmbientOcclusion, CameraShake, ChromaticAberration, ColorGrading, ColorLut, Decal,
    DepthOfField, FilmGrain, Glow, GodRays, Light2d, Lighting2d, MotionBlur, Occluder2d, Outline,
    ScreenFlash, SpriteFx, Vignette,
};
use crate::material::StandardMaterial;
use crate::particles::{EmitterHandle, ParticleEmitter, ParticleSystem};
use crate::ribbons::{Beam, Ribbon, Trail, TrailStyle, TrailStyleHandle};
use crate::skin::{JointPalette, MorphTargets, SkinData, SkinnedInstance};
use crate::texture::TextureUpload;
use glam::{Mat4, Vec3};
use std::sync::atomic::{AtomicU64, Ordering};

/// One mesh vertex. Positions and normals are in the mesh's local space.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// Vertex colour, multiplied into the base colour (white when unused).
    pub color: [f32; 4],
    /// The tangent frame normal maps are read in (glTF's): `xyz` the
    /// tangent, along +u; `w` the sign of the bitangent cross(normal,
    /// tangent) × w, which points up the image (towards -v, as uvs run down
    /// it), so a map's red tilts towards +u and its green up the image. `w`
    /// 0 means none: the renderer generates it from the positions, normals
    /// and uvs ([`MeshData::generate_tangents`]).
    pub tangent: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [0.0; 2],
            color: [1.0; 4],
            tangent: [0.0; 4],
        }
    }
}

/// How a mesh's vertices are held on the GPU
/// ([`RenderScene::set_vertex_encoding`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VertexEncoding {
    /// [`Vertex`] as given: 64 bytes a vertex in f32, for every pipeline.
    #[default]
    Full,
    /// 16 bytes a vertex where the mesh is drawn with a plain material (the
    /// lean pipelines; README "Instance and vertex encodings"): the
    /// position quantised to 16 bits per axis over the mesh's bounds, the
    /// normal octahedral in two 16-bit snorms, the colour in 8 bits a
    /// channel. Quantised, so its pixels are close to [`VertexEncoding::Full`]'s
    /// but not the same. The f32 vertices are uploaded too only if a
    /// material that is not plain, a scatter or a skinned instance draws
    /// the mesh. Ignored for skinned and morphed meshes.
    Compact,
}

/// Triangle-list geometry: vertices and `u32` indices (three per triangle).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Whether some vertex has no tangent (`tangent[3] == 0`).
    pub fn lacks_tangents(&self) -> bool {
        self.vertices.iter().any(|v| v.tangent[3] == 0.0)
    }

    /// Gives every vertex without a tangent one: the uv gradient of the
    /// triangles around it (Lengyel's method), made orthogonal to its normal,
    /// with the sign that makes the bitangent point towards -v ([`Vertex::tangent`]);
    /// a vertex whose triangles have no usable uvs gets any tangent
    /// orthogonal to its normal. Vertices that have a tangent (the cook's
    /// MikkTSpace ones) keep it.
    pub fn generate_tangents(&mut self) {
        if !self.lacks_tangents() {
            return;
        }
        let n = self.vertices.len();
        let mut along_u = vec![Vec3::ZERO; n];
        let mut along_v = vec![Vec3::ZERO; n];
        for tri in self.indices.as_chunks::<3>().0 {
            let [a, b, c] = tri.map(|i| i as usize);
            if a >= n || b >= n || c >= n {
                continue;
            }
            let (va, vb, vc) = (&self.vertices[a], &self.vertices[b], &self.vertices[c]);
            let p0 = Vec3::from(va.position);
            let e1 = Vec3::from(vb.position) - p0;
            let e2 = Vec3::from(vc.position) - p0;
            let (du1, dv1) = (vb.uv[0] - va.uv[0], vb.uv[1] - va.uv[1]);
            let (du2, dv2) = (vc.uv[0] - va.uv[0], vc.uv[1] - va.uv[1]);
            let r = du1 * dv2 - du2 * dv1;
            if r.abs() < 1e-12 {
                continue;
            }
            let su = (e1 * dv2 - e2 * dv1) / r;
            let sv = (e2 * du1 - e1 * du2) / r;
            for i in [a, b, c] {
                along_u[i] += su;
                along_v[i] += sv;
            }
        }
        for (i, v) in self.vertices.iter_mut().enumerate() {
            if v.tangent[3] != 0.0 {
                continue;
            }
            let normal = Vec3::from(v.normal).normalize_or(Vec3::Y);
            let mut t = along_u[i] - normal * normal.dot(along_u[i]);
            if t.length_squared() < 1e-20 {
                // No uv gradient: any direction across the normal.
                t = normal.any_orthonormal_vector();
            }
            let t = t.normalize();
            // The bitangent up the image: against the uv's v.
            let w = if normal.cross(t).dot(along_v[i]) > 0.0 {
                -1.0
            } else {
                1.0
            };
            v.tangent = t.extend(w).to_array();
        }
    }

    /// Axis-aligned bounds of the vertex positions (`None` for an empty mesh).
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self.vertices.iter().map(|v| Vec3::from(v.position));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }
}

/// How the texel values of a colour texture are encoded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// sRGB-encoded colour (base colour, emissive): decoded to linear when sampled.
    #[default]
    Srgb,
    /// Linear data (normal maps, metallic-roughness).
    Linear,
}

/// Whether the renderer builds a mip chain for a texture when it uploads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mips {
    /// Generate every level down to 1x1 on the GPU (box filter, linear space).
    #[default]
    Generate,
    /// Upload level 0 only.
    None,
}

/// An RGBA8 image, rows top to bottom, `width * height * 4` bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub color_space: ColorSpace,
    pub mips: Mips,
}

/// A texture as a host hands it over: RGBA8 (mips generated on the GPU) or
/// pre-encoded in one of [`crate::RenderInfo::texture_formats`]
/// ([`RenderScene::add_texture_source`]).
#[derive(Clone, Debug, PartialEq)]
pub enum TextureSource {
    Rgba(TextureData),
    Encoded(TextureUpload),
    /// A render target's colour or depth ([`crate::RenderScene::add_render_target`]):
    /// drawn on the GPU by a [`crate::View`], never uploaded.
    Target(crate::TargetTexture),
}

impl TextureSource {
    /// Width and height of level 0.
    pub fn size(&self) -> (u32, u32) {
        match self {
            TextureSource::Rgba(t) => (t.width, t.height),
            TextureSource::Encoded(t) => (t.desc.width, t.desc.height),
            TextureSource::Target(t) => (t.width, t.height),
        }
    }
}

/// How a material treats base-colour alpha.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AlphaMode {
    /// Alpha is ignored.
    #[default]
    Opaque,
    /// Fragments whose alpha is below `cutoff` are discarded (glTF `MASK`).
    Mask { cutoff: f32 },
    /// Alpha blended over the opaque scene, sorted back to front (glTF
    /// `BLEND`; [`crate::StandardMaterial::transparent`]).
    Blend,
}

/// A metallic-roughness PBR material (the glTF 2.0 model). Every factor
/// multiplies its texture; a missing texture behaves as white (or as a flat
/// normal). The default is an opaque, white, non-metallic, half-rough surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Linear RGBA base colour factor.
    pub base_color: [f32; 4],
    pub base_color_texture: Option<TextureHandle>,
    pub metallic: f32,
    pub roughness: f32,
    /// Metalness in blue, roughness in green (glTF layout).
    pub metallic_roughness_texture: Option<TextureHandle>,
    /// Tangent-space normal map; the tangent frame comes from screen derivatives.
    pub normal_texture: Option<TextureHandle>,
    pub normal_scale: f32,
    /// Linear emitted radiance (before the emissive texture).
    pub emissive: [f32; 3],
    pub emissive_texture: Option<TextureHandle>,
    pub alpha_mode: AlphaMode,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: [1.0; 4],
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            metallic_roughness_texture: None,
            normal_texture: None,
            normal_scale: 1.0,
            emissive: [0.0; 3],
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

/// A handle's slot is its low 24 bits; the top 8 are the slot's generation.
const INDEX_BITS: u32 = 24;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;

/// Which slots of one kind of resource hold one, and their generations:
/// `remove_*` empties a slot and moves its generation on, and the next
/// `add_*` of the kind fills it again (last freed first), so handles are
/// reused without an old one reaching the new resource.
#[derive(Debug, Default)]
struct Slots {
    generation: Vec<u8>,
    removed: Vec<bool>,
    free: Vec<u32>,
    /// Whether any slot of the kind was ever removed (no check is needed
    /// before).
    any_removed: bool,
}

impl Slots {
    /// The slot to fill: a freed one, or a new one at `len` (the kind's
    /// length). Returns the slot, its generation and whether it was reused.
    fn take(&mut self, len: usize, reuse: bool) -> (usize, u8, bool) {
        if reuse && let Some(i) = self.free.pop() {
            let i = i as usize;
            self.removed[i] = false;
            return (i, self.generation[i], true);
        }
        debug_assert_eq!(len, self.generation.len());
        self.generation.push(0);
        self.removed.push(false);
        (len, 0, false)
    }

    fn live(&self, index: usize, generation: u8) -> bool {
        index < self.generation.len()
            && self.generation[index] == generation
            && !self.removed[index]
    }

    /// Empties a live slot; `reuse`: a later add may fill it.
    fn remove(&mut self, index: usize, reuse: bool) {
        self.removed[index] = true;
        self.generation[index] = self.generation[index].wrapping_add(1);
        self.any_removed = true;
        if reuse {
            self.free.push(index as u32);
        }
    }

    fn generation(&self, index: usize) -> u8 {
        self.generation.get(index).copied().unwrap_or(0)
    }
}

macro_rules! handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u32);
        impl $name {
            /// The position of the resource in its scene: its slot, in
            /// insertion order (a slot freed by a `remove_*` is reused).
            pub fn index(self) -> usize {
                (self.0 & INDEX_MASK) as usize
            }
            /// How many times the slot was emptied before this handle was
            /// given (wrapping at 256): a handle of a removed resource no
            /// longer matches its slot.
            pub fn generation(self) -> u8 {
                (self.0 >> INDEX_BITS) as u8
            }
            #[allow(dead_code)]
            pub(crate) fn from_index(index: usize) -> Self {
                $name(index as u32)
            }
            #[allow(dead_code)]
            pub(crate) fn from_parts(index: usize, generation: u8) -> Self {
                debug_assert!(index as u32 <= INDEX_MASK);
                $name(index as u32 | (generation as u32) << INDEX_BITS)
            }
        }
    };
}
handle!(
    /// A mesh added to a [`RenderScene`].
    MeshHandle
);
handle!(
    /// A material added to a [`RenderScene`].
    MaterialHandle
);
handle!(
    /// A texture added to a [`RenderScene`].
    TextureHandle
);
handle!(
    /// A material shader added to a [`RenderScene`].
    ShaderHandle
);

impl TextureHandle {
    /// The handle as a [`crate::StandardMaterial`] texture slot: its index
    /// plus one (0 is "no texture").
    pub fn slot(self) -> u32 {
        self.0 + 1
    }

    /// The handle a texture slot names (`None` for slot 0).
    pub fn from_slot(slot: u32) -> Option<TextureHandle> {
        slot.checked_sub(1).map(TextureHandle)
    }
}
handle!(
    /// An MSDF font added to a [`RenderScene`].
    FontHandle
);
handle!(
    /// Terrain data added to a [`RenderScene`] ([`RenderScene::add_terrain`]).
    TerrainHandle
);
handle!(
    /// A prefiltered environment map added to a [`RenderScene`]
    /// ([`RenderScene::add_environment_map`]).
    EnvironmentMapHandle
);
handle!(
    /// A texture array added to a [`RenderScene`]
    /// ([`RenderScene::add_texture_array`]).
    TextureArrayHandle
);

impl TextureArrayHandle {
    /// The handle as a [`crate::StandardMaterial`] texture-array slot
    /// (`mapArray`, `normalMapArray`): its bits plus one (0 is none).
    pub fn slot(self) -> u32 {
        self.0 + 1
    }

    /// The handle a texture-array slot names (`None` for slot 0).
    pub fn from_slot(slot: u32) -> Option<TextureArrayHandle> {
        slot.checked_sub(1).map(TextureArrayHandle)
    }
}
handle!(
    /// A colour LUT added to a [`RenderScene`] ([`RenderScene::add_lut`]).
    LutHandle
);
handle!(
    /// A colour palette added to a [`RenderScene`] for
    /// [`crate::Stylise`]'s quantisation ([`RenderScene::add_palette`]).
    PaletteHandle
);

/// A sprite drawn from a layer of a texture array instead of its texture
/// ([`RenderScene::sprite_arrays`], [`Overlay::sprite_arrays`]): a sheet of
/// same-sized frames batched into one draw whichever frame each sprite
/// shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteArray {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    pub array: TextureArrayHandle,
    /// The layer drawn, below the array's layer count.
    pub layer: u32,
}

/// One object to draw: a mesh with a material at a world transform. Callers
/// submit instances one by one; the renderer batches all instances that share
/// a (mesh, material) pair and caster flag into one instanced draw per pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    /// Local to world. Normals use the upper 3x3 directly, so keep scale
    /// uniform. Affine: the GPU reads the first three rows, the last is
    /// taken as (0, 0, 0, 1).
    pub transform: Mat4,
    /// The texture-array layer this instance's material samples
    /// ([`crate::StandardMaterial::mapArray`] and `normalMapArray`), in
    /// place of the material's `mapLayer` and `normalMapLayer`: variants of
    /// one material (tiles, skins) stay one batch, one draw. `None`: the
    /// material's layers. A material without array maps ignores it.
    pub material_layer: Option<u32>,
    /// Its own hit flash and tint on top of its material's look, carried in
    /// the instance record (one material, one draw, whatever each instance
    /// shows): a glTF model's instances share their materials.
    pub fx: InstanceFx,
    /// Drawn into the shadow maps (the sun's cascades and the shadowed
    /// local lights'). `false` keeps it out of every shadow pass while it
    /// still receives shadows: a lamp's own glass around its light, a glow
    /// card, a decal-like slab. [`RenderScene::add_instance`] casts.
    pub cast_shadows: bool,
    /// Darkened by the shadow maps. `false` lights it as if nothing stood
    /// between it and any light (the sun's cascades and every shadowed
    /// local light are not read for it), while it still casts as
    /// `cast_shadows` says: a glowing sign, a hero kept readable in a dark
    /// alley. Carried in the instance record (`instances.rs`), so it never
    /// splits a batch. [`RenderScene::add_instance`] receives.
    pub receive_shadows: bool,
}

/// An instance's own look, beside its transform in the instance record and
/// read by the lit shaders on every tier (pbr_fs.wgsl, physical.wgsl):
/// the hit flash mixes the lit colour towards `flash_color`, the tint
/// multiplies the material's base colour. Colours travel as 8 bits a
/// channel. [`InstanceFx::NONE`] (the default) shades exactly as the
/// material does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceFx {
    /// The hit flash, 0 to 1: the lit colour mixed towards `flash_color` by
    /// this much (0: none).
    pub flash: f32,
    /// The flash's linear colour.
    pub flash_color: Vec3,
    /// A linear colour, 0 to 1, multiplied with the material's base colour
    /// (white: none).
    pub tint: Vec3,
}

impl InstanceFx {
    /// No flash, a white tint: the material's own look.
    pub const NONE: InstanceFx = InstanceFx {
        flash: 0.0,
        flash_color: Vec3::ONE,
        tint: Vec3::ONE,
    };
}

impl Default for InstanceFx {
    fn default() -> Self {
        InstanceFx::NONE
    }
}

/// How a [`Camera`] projects the world onto the target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// Perspective with a vertical field of view in degrees.
    Perspective { fov_y_degrees: f32 },
    /// Orthographic: `height` world units fill the target vertically; the
    /// width follows the target's aspect ratio. The 2D camera mode.
    Orthographic { height: f32 },
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective {
            fov_y_degrees: 50.0,
        }
    }
}

/// How bright the HDR image is made before tonemapping. One EV is one stop:
/// +1 doubles the image, -1 halves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Exposure {
    /// A fixed exposure: the image is multiplied by `2^ev` (0 leaves it as lit).
    Manual { ev: f32 },
    /// Metered from the frame being drawn: the log-average luminance is
    /// brought to middle grey (0.18), then `compensation` is added and the
    /// result is clamped to `min_ev..=max_ev`.
    ///
    /// With both speeds 0 (the default) that is the exposure: it reads only
    /// the current frame, so the same frame always gets the same exposure.
    /// Otherwise the exposure adapts over time like an eye: it moves towards
    /// the metered one by at most `adapt_speed_up` EV per second while
    /// rising (the view got darker) and `adapt_speed_down` while falling
    /// (it got brighter); a speed of 0 is instant in its direction. Time is
    /// the scene's ticks ([`crate::RenderScene::tick`] over
    /// [`crate::RenderScene::tick_rate`]) since the previous adapted frame,
    /// never the wall clock, so the same ticks give the same frames; the
    /// first adapted frame (or one whose tick went back) starts at the
    /// metered exposure. The adapted exposure is renderer state that
    /// [`crate::Renderer::save_state`] and [`crate::Renderer::restore_state`]
    /// carry across a world snapshot.
    Auto {
        compensation: f32,
        min_ev: f32,
        max_ev: f32,
        adapt_speed_up: f32,
        adapt_speed_down: f32,
    },
}

impl Exposure {
    /// A manual exposure from a linear multiplier (`linear(0.8)` scales the
    /// image by 0.8).
    pub fn linear(multiplier: f32) -> Exposure {
        Exposure::Manual {
            ev: multiplier.max(1e-12).log2(),
        }
    }

    /// Auto exposure with no compensation over a wide range (-10..=10 EV),
    /// instant (no adaptation over time).
    pub fn auto() -> Exposure {
        Exposure::Auto {
            compensation: 0.0,
            min_ev: -10.0,
            max_ev: 10.0,
            adapt_speed_up: 0.0,
            adapt_speed_down: 0.0,
        }
    }

    /// The adaptation speeds (EV per second up, down) when this exposure
    /// adapts over time: auto with either speed above 0.
    pub fn adaptation(self) -> Option<(f32, f32)> {
        match self {
            Exposure::Auto {
                adapt_speed_up: up,
                adapt_speed_down: down,
                ..
            } if up > 0.0 || down > 0.0 => Some((up.max(0.0), down.max(0.0))),
            _ => None,
        }
    }
}

impl Default for Exposure {
    fn default() -> Self {
        Exposure::Manual { ev: 0.0 }
    }
}

/// A camera looking from `position` at `target` (right-handed, y up by
/// default, depth 0..1). The aspect ratio is the render target's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: Projection,
    pub near: f32,
    pub far: f32,
    pub exposure: Exposure,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(0.0, 2.0, 6.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::default(),
            near: 0.1,
            far: 1000.0,
            exposure: Exposure::default(),
        }
    }
}

impl Camera {
    /// The 2D camera: orthographic, looking down -z at `center` on the z = 0
    /// plane, `height` world units tall, y up. Sprites and text at z = 0 face it.
    pub fn orthographic_2d(center: glam::Vec2, height: f32) -> Camera {
        Camera {
            position: center.extend(100.0),
            target: center.extend(0.0),
            up: Vec3::Y,
            projection: Projection::Orthographic { height },
            near: 0.0,
            far: 200.0,
            exposure: Exposure::default(),
        }
    }
}

/// What the directional light's cascaded shadow maps must cover.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ShadowBounds {
    /// The world bounds of every submitted instance: cascades reach from the
    /// camera's near plane to the far side of those bounds, and every instance
    /// in them can cast.
    #[default]
    Auto,
    /// A sphere around `center` with `radius`: cascades reach to its far side
    /// and casters inside it are kept.
    Sphere { center: Vec3, radius: f32 },
}

/// The sun: one directional light with three cascaded shadow maps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionalLight {
    /// Points from the scene towards the light (need not be normalized).
    pub direction_to_light: Vec3,
    /// Linear colour; the radiance is `color * intensity`.
    pub color: Vec3,
    pub intensity: f32,
    pub cast_shadows: bool,
    pub shadow_bounds: ShadowBounds,
    /// How far from the camera shadows reach (they fade out over the last
    /// tenth). The cascades cover the nearer of this and the far side of
    /// `shadow_bounds`, so a smaller distance means sharper shadows.
    pub max_shadow_distance: f32,
    /// Soft shadows (PCSS): the penumbra's width per metre between caster
    /// and receiver, the tangent of the sun's apparent radius (the real
    /// sun's is 0.005). 0, the default, is the hard 1-texel PCF edge;
    /// 0.01 to 0.05 is a soft sun. Each shadowed pixel then searches the
    /// cascade for blockers and filters over the penumbra they give
    /// (16 + 64 texel reads instead of 4).
    pub shadow_softness: f32,
    /// The fraction of each cascade's depth over which it fades into the
    /// next one, hiding the seam where the shadow's resolution changes (0,
    /// the default: none; 0.1 to 0.2 typical). The next cascade then also
    /// covers that band, and its pixels read two cascades.
    pub cascade_blend: f32,
    /// One shadow map over a fixed box instead of the cascades (0, the
    /// default: the cascades): its half size in metres across the light,
    /// centred `shadow_focus` metres ahead of the camera along its level
    /// forward, at the camera's height less `shadow_focus_drop`. Everything
    /// outside the box is lit. three.js games that fit a directional light's
    /// orthographic shadow camera to a walker draw this way; it also draws
    /// the casters once instead of once per cascade.
    pub shadow_box: f32,
    pub shadow_focus: f32,
    pub shadow_focus_drop: f32,
    /// With `shadow_box`: the depth bias and the normal offset in metres
    /// (0: one texel each).
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        DirectionalLight {
            direction_to_light: Vec3::new(0.45, 0.8, 0.35),
            color: Vec3::ONE,
            intensity: 3.0,
            cast_shadows: true,
            shadow_bounds: ShadowBounds::Auto,
            max_shadow_distance: 200.0,
            shadow_softness: 0.0,
            cascade_blend: 0.0,
            shadow_box: 0.0,
            shadow_focus: 0.0,
            shadow_focus_drop: 0.0,
            shadow_bias: 0.0,
            shadow_normal_bias: 0.0,
        }
    }
}

/// The most local lights one frame draws, after those outside the view are
/// culled: past it the least important are dropped (the radiance at the
/// camera; `clusters.rs`) and counted in [`crate::FrameStats::lights_dropped`].
pub const MAX_LIGHTS: usize = 256;
pub use crate::shadows::{MAX_SHADOWED_LIGHTS, NO_COMPUTE_SHADOWED_LIGHTS};

/// The shape of a local light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightKind {
    /// Shines in every direction (three.js `PointLight`).
    Point,
    /// A cone along `direction` (three.js `SpotLight`): `angle_degrees` is the
    /// half-angle of the outer cone, `penumbra` (0..1) the fraction of it over
    /// which the light fades out.
    Spot {
        direction: Vec3,
        angle_degrees: f32,
        penumbra: f32,
    },
    /// A rectangle of light (three.js `RectAreaLight`, `area.rs`): centred
    /// on `position`, spanning `±half_width` and `±half_height` (world
    /// vectors along its sides), emitting `color * intensity` as radiance
    /// along `-(half_width × half_height)`, and from its back as well when
    /// `two_sided`. Shaded with linearly transformed cosines; with
    /// `cast_shadows` it takes one local shadow layer (two when two-sided):
    /// a [`crate::RECT_SHADOW_FOV`] view from its centre along its normal,
    /// read with a penumbra that widens with the rectangle's size (area.rs).
    /// It fades to zero at `range` as the other lights do.
    Rect {
        half_width: Vec3,
        half_height: Vec3,
        two_sided: bool,
    },
}

/// A point or spot light. Its radiance at distance `d` is
/// `color * intensity / d²`, faded smoothly to zero at `range`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub position: Vec3,
    /// Linear colour.
    pub color: Vec3,
    /// Luminous intensity (candela-like: the radiance at 1 m).
    pub intensity: f32,
    /// The distance at which the light reaches zero.
    pub range: f32,
    /// Shadows from this light (cube faces or two paraboloid hemispheres for
    /// a point light, [`crate::PointShadows`]; one map for a spot). Of the
    /// shadowed lights in view, the tier's [`crate::RenderInfo::shadowed_lights`]
    /// with the most radiance at the camera get them (8 on the full tier, 4
    /// on WebGL2); the rest are counted in
    /// [`crate::FrameStats::shadowed_lights_dropped`].
    pub cast_shadows: bool,
}

impl Default for Light {
    fn default() -> Self {
        Light {
            kind: LightKind::Point,
            position: Vec3::new(0.0, 2.0, 0.0),
            color: Vec3::ONE,
            intensity: 10.0,
            range: 10.0,
            cast_shadows: false,
        }
    }
}

/// The procedural sky the environment lighting is generated from: a zenith
/// to horizon gradient, dark ground, and a sun disc with a glow. A change
/// regenerates the image-based light (a few passes; a day-night cycle
/// changes it in steps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProceduralSky {
    /// Where the sun sits on the sky (from the origin towards the sun).
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    /// The sky's colours, disc, glow and stars (`None`: the built-in sky).
    pub look: Option<crate::SkyLook>,
}

impl Default for ProceduralSky {
    fn default() -> Self {
        ProceduralSky {
            sun_direction: Vec3::new(0.45, 0.8, 0.35),
            sun_color: Vec3::new(1.0, 0.95, 0.85),
            look: None,
        }
    }
}

/// What fills the pixels no geometry covers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Background {
    /// The sky cubemap the image-based lighting comes from.
    #[default]
    Sky,
    /// A flat linear HDR colour (exposed and tonemapped like the scene).
    Color(Vec3),
    /// The procedural sky drawn per pixel with the environment sky's look
    /// (or the built-in colours), its sun at `sun_direction` with
    /// `sun_color`: the sun of a day-night cycle moves smoothly while the
    /// image-based light follows in steps. Stars are drawn here only.
    Procedural {
        sun_direction: Vec3,
        sun_color: Vec3,
    },
}

/// Environment lighting and background.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    pub sky: ProceduralSky,
    /// Strength of the image-based (ambient) light.
    pub intensity: f32,
    pub background: Background,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            sky: ProceduralSky::default(),
            intensity: 0.55,
            background: Background::Sky,
        }
    }
}

/// The tonemapping curve from linear HDR to the display range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tonemap {
    /// ACES (the fitted curve).
    #[default]
    Aces,
    /// None: the exposed linear colour is clamped to 0..1. With exposure 0 EV,
    /// sprite and text colours come out exactly as authored (the 2D choice).
    None,
    /// AgX (Troy Sobotka's; the polynomial fit and Rec.2020 inset and outset
    /// that Blender 4, Filament and three.js `AgXToneMapping` use, base look,
    /// no "punchy"): highlights desaturate towards white instead of skewing
    /// hue, and saturated lights keep their hue further up than under ACES.
    Agx,
    /// Khronos PBR Neutral (three.js `NeutralToneMapping`): colours whose
    /// brightest channel stays under 0.76 pass through, less the 0.04 of a
    /// dielectric's reflection (so a white-lit base colour shows as
    /// authored), and only brighter ones roll off towards 1, desaturating
    /// to white. For product shots and saturated stylised colour, where
    /// ACES's contrast and hue shifts are unwanted.
    Neutral,
}

/// Bloom: light above a threshold bleeds into its surroundings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bloom {
    /// Brightness (after exposure) where bloom starts; 1.0 is display white.
    pub threshold: f32,
    /// Softens the threshold over `threshold ± knee` (0 = a hard cut).
    pub knee: f32,
    /// How much of the blurred light is added back.
    pub intensity: f32,
}

impl Default for Bloom {
    fn default() -> Self {
        Bloom {
            threshold: 1.0,
            knee: 0.5,
            intensity: 0.6,
        }
    }
}

/// Post-processing around the lit image. Every effect is `None` (off, no
/// pass recorded) by default. Exposure belongs to the [`Camera`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PostEffects {
    /// `None` turns bloom off (no passes run).
    pub bloom: Option<Bloom>,
    pub tonemap: Tonemap,
    /// Screen-space ambient occlusion (GTAO / SSAO).
    pub ao: Option<AmbientOcclusion>,
    /// Outlines for every opaque surface (materials can set their own).
    pub outline: Option<Outline>,
    pub depth_of_field: Option<DepthOfField>,
    pub motion_blur: Option<MotionBlur>,
    pub god_rays: Option<GodRays>,
    /// The 2D glow buffer (sprites with [`SpriteFx::glow`]).
    pub glow: Option<Glow>,
    pub color_grading: Option<ColorGrading>,
    /// Colour adjustments after the grading (baked with its table).
    pub color_adjust: Option<crate::ColorAdjust>,
    pub vignette: Option<Vignette>,
    pub chromatic_aberration: Option<ChromaticAberration>,
    /// Barrel or pincushion distortion (a pass after the stylisation
    /// passes that read the depth).
    pub lens_distortion: Option<crate::LensDistortion>,
    pub film_grain: Option<FilmGrain>,
    /// Contrast-adaptive sharpening of the display image (three.js-style CAS
    /// over the graded, sRGB-encoded colour), its strength 0 to 1.
    pub sharpen: Option<f32>,
    /// A full-screen hit flash.
    pub flash: Option<ScreenFlash>,
    /// Screen-space reflections (the full tier; `None`: off, and WebGL2
    /// draws none).
    pub ssr: Option<crate::ScreenSpaceReflections>,
    /// `Some(radius)`: [`PostEffects::bloom`] as three.js UnrealBloomPass
    /// (luminance high pass, five Gaussian mips mixed by `radius`, the knee
    /// as its smoothWidth and the intensity as its strength); `None`: the
    /// soft-knee, 13-tap chain.
    pub bloom_unreal: Option<f32>,
    /// With [`PostEffects::bloom_unreal`]: the per-channel contrast
    /// `0.18 · (c / 0.18)^k` the high pass sees (1 or 0 = none).
    pub bloom_contrast: f32,
}

impl PostEffects {
    /// The colour adjustments that change something: the tonemap then
    /// samples their baked table instead of the grading's.
    pub(crate) fn adjust(&self) -> Option<crate::ColorAdjust> {
        self.color_adjust.filter(|a| !a.is_identity())
    }
}

/// A diagnostic view in place of the lit image.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DebugView {
    #[default]
    None,
    /// Each lit pixel coloured by the shadow cascade that covers it: red,
    /// green, blue for cascades 0, 1, 2; black beyond the last one.
    Cascades,
    /// The screen-space ambient occlusion term (white: unoccluded), with
    /// [`PostEffects::ao`] set.
    AmbientOcclusion,
    /// The lit materials' base colour (their albedo after maps and hooks),
    /// unlit.
    Albedo,
    /// The diffuse image-based light each lit material receives.
    IblDiffuse,
    /// The specular image-based light each lit material reflects.
    IblSpecular,
    /// The direct light (directional, point and spot) each lit material
    /// receives, shadows applied.
    DirectLight,
    /// The depth the ambient occlusion pass reads, as its fraction of a
    /// metre (one ramp per metre), in the AO view; with [`PostEffects::ao`]
    /// set.
    AoDepth,
    /// The ambient occlusion term before its depth-aware blur; with
    /// [`PostEffects::ao`] set.
    AoRaw,
}

impl DebugView {
    /// `FrameUniform.light_color.w`: which view the lit shaders draw
    /// (0 none, 1 cascades, 2 ambient occlusion, 3 albedo, 4 diffuse IBL,
    /// 5 specular IBL, 6 direct light).
    pub(crate) fn lit_code(self) -> f32 {
        match self {
            DebugView::None => 0.0,
            DebugView::Cascades => 1.0,
            DebugView::AmbientOcclusion | DebugView::AoDepth | DebugView::AoRaw => 2.0,
            DebugView::Albedo => 3.0,
            DebugView::IblDiffuse => 4.0,
            DebugView::IblSpecular => 5.0,
            DebugView::DirectLight => 6.0,
        }
    }

    /// `FxPostUniform.cam.w`: what the AO pass writes in place of its term
    /// (0 the term, 1 the depth's fraction, 2 the term unblurred).
    pub(crate) fn ao_code(self) -> f32 {
        match self {
            DebugView::AoDepth => 1.0,
            DebugView::AoRaw => 2.0,
            _ => 0.0,
        }
    }
}

/// A textured quad in the world's xy plane, facing +z (towards the 2D
/// camera). Callers submit sprites one by one; the renderer sorts them by
/// (`layer`, texture) and draws each run in one instanced draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite {
    /// `None` draws a solid quad of `color`.
    pub texture: Option<TextureHandle>,
    /// World position of the anchor point.
    pub position: Vec3,
    /// Width and height in world units.
    pub size: glam::Vec2,
    /// The point of the quad at `position`, as a fraction of its size from the
    /// bottom-left corner: (0.5, 0.5) is the centre.
    pub anchor: glam::Vec2,
    /// Counter-clockwise rotation around the anchor, in radians.
    pub rotation: f32,
    /// Linear RGBA multiplied with the texture (alpha blends).
    pub color: [f32; 4],
    /// The texture region drawn: `[u0, v0, u1, v1]` with v down (the image's
    /// first row is v = 0). The default is the whole texture.
    pub uv_rect: [f32; 4],
    /// Draw order: higher layers draw over lower ones. Within a layer, sprites
    /// are grouped by texture, then drawn in submission order.
    pub layer: i32,
    /// In [`Overlay::sprites`]: the index of the clip rectangle in
    /// [`Overlay::clips`] the sprite is cut to (`None`: the whole output).
    /// Ignored for world sprites.
    pub clip: Option<u32>,
}

impl Sprite {
    /// The same quad with its anchor point moved to `pivot` (x and y; the
    /// sprite keeps its z): the anchor is shifted by the offset in the
    /// sprite's own rotated frame, so every corner stays where it was. A
    /// faced sprite turns about its anchor, so the pieces of one faced
    /// nine-slice ([`crate::nine_slice`]) are anchored at the panel's anchor to
    /// turn as one panel. An axis of zero size keeps its anchor.
    pub fn anchored_at(&self, pivot: Vec3) -> Sprite {
        let d = pivot.truncate() - self.position.truncate();
        let (sin, cos) = self.rotation.sin_cos();
        // The offset in the quad's frame (the rotation undone).
        let local = glam::Vec2::new(d.x * cos + d.y * sin, -d.x * sin + d.y * cos);
        let shift = |l: f32, size: f32| if size != 0.0 { l / size } else { 0.0 };
        Sprite {
            position: Vec3::new(pivot.x, pivot.y, self.position.z),
            anchor: self.anchor
                + glam::Vec2::new(shift(local.x, self.size.x), shift(local.y, self.size.y)),
            ..*self
        }
    }
}

/// How a world sprite is turned towards the camera that draws it
/// ([`RenderScene::sprite_facing`]): a billboard. Computed in the sprite
/// vertex shader from each camera's own axes (the main camera's, a
/// [`crate::View`]'s, a reflection probe face's), so it never feeds back
/// into anything the simulation reads. The sprite turns about its anchor;
/// `rotation` still turns it within its own plane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpriteFacing {
    /// In the world's xy plane, facing +z (every sprite until billboards).
    #[default]
    None,
    /// In the camera's view plane (three.js `Sprite`, the particle
    /// billboards): its x along the camera's right, its y along its up.
    Camera,
    /// Upright, turned about the world's y axis towards the camera: its y
    /// stays +y, its x the camera's right laid level. Trees, characters
    /// and props that must not lean back when the camera looks down.
    CameraY,
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            texture: None,
            position: Vec3::ZERO,
            size: glam::Vec2::ONE,
            anchor: glam::Vec2::splat(0.5),
            rotation: 0.0,
            color: [1.0; 4],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            layer: 0,
            clip: None,
        }
    }
}

/// One glyph of an MSDF atlas (the layout `msdf-atlas-gen` writes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub codepoint: char,
    /// Horizontal advance in em.
    pub advance: f32,
    /// The quad around the glyph relative to the pen on the baseline, in em,
    /// y up: `[left, bottom, right, top]`. All zero for blank glyphs (space).
    pub plane_bounds: [f32; 4],
    /// The glyph's region in the atlas in pixels, y down from the first row:
    /// `[left, top, right, bottom]`.
    pub atlas_bounds: [f32; 4],
}

/// A kerning pair of an [`MsdfFont`]: `adjust` em added to `left`'s advance
/// when `right` follows it (negative pulls the two together).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kern {
    pub left: char,
    pub right: char,
    pub adjust: f32,
}

/// A multi-channel signed distance field font: an atlas texture plus its
/// glyph table. The atlas is produced ahead of time (the cook's job); it must
/// be an RGB(A) MSDF uploaded with [`ColorSpace::Linear`] and [`Mips::None`].
#[derive(Clone, Debug, PartialEq)]
pub struct MsdfFont {
    pub atlas: TextureHandle,
    /// The distance field's range in atlas pixels (`msdf-atlas-gen -pxrange`).
    pub distance_range: f32,
    /// Baseline-to-baseline distance in em.
    pub line_height: f32,
    pub glyphs: Vec<Glyph>,
    /// Pair kerning (the cook's `kerning` table), applied between adjacent
    /// characters of a line; empty for a font that does not kern.
    pub kerning: Vec<Kern>,
}

impl MsdfFont {
    /// The width of one line of `text` in em as the renderer lays it out
    /// (advances and kerning; characters the font lacks take no space).
    pub fn measure(&self, text: &str) -> f32 {
        let mut w = 0.0f32;
        let mut prev: Option<char> = None;
        for ch in text.chars() {
            let Some(g) = self.glyphs.iter().find(|g| g.codepoint == ch) else {
                prev = None;
                continue;
            };
            if let Some(k) = prev.and_then(|p| {
                self.kerning
                    .iter()
                    .find(|k| k.left == p && k.right == ch)
                    .map(|k| k.adjust)
            }) {
                w += k;
            }
            w += g.advance;
            prev = Some(ch);
        }
        w
    }
}

/// Where a shaper put one character of a [`Text`] ([`Text::placements`]),
/// in em: the pen moves `advance` after it, and its glyph is drawn offset by
/// `(x, y)` from the pen (y up), as HarfBuzz's positions are.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GlyphPlacement {
    pub advance: f32,
    pub x: f32,
    pub y: f32,
}

/// A run of text drawn with an MSDF font, laid out left to right from the
/// pen at `position` (the first line's baseline), lines split at `\n`.
/// Characters missing from the font are skipped and counted in
/// [`crate::FrameStats::missing_glyphs`].
#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub font: FontHandle,
    pub text: String,
    pub position: Vec3,
    /// Em size in world units.
    pub size: f32,
    /// Counter-clockwise rotation around `position`, in radians.
    pub rotation: f32,
    /// Linear RGBA.
    pub color: [f32; 4],
    /// Sorted together with sprites: higher layers draw over lower ones.
    pub layer: i32,
    /// In [`Overlay::texts`]: the index of the clip rectangle in
    /// [`Overlay::clips`] the text is cut to, per pixel (`None`: the whole
    /// output). Ignored for world text.
    pub clip: Option<u32>,
    /// Added to every glyph's advance, in em (CSS `letter-spacing`: after
    /// each character, the last included): a type ramp's tracking. 0 is the
    /// font's own spacing.
    pub tracking: f32,
    /// In [`Overlay::texts`]: thickens the glyphs by this many CSS pixels at
    /// their edge (a faux bold from the distance field; 0 is the font's
    /// weight). Ignored for world text.
    pub weight: f32,
    /// In [`Overlay::texts`]: a soft glow around the glyphs read from the
    /// font's true distance field (MTSDF alpha), drawn under every glyph of
    /// the text. Ignored for world text (it has [`TextOutline`]).
    pub glow: Option<TextGlow>,
    /// Positions from a shaper, one per character of `text` (`\n` included):
    /// each replaces its character's advance, kerning and tracking, and
    /// offsets its glyph (marks over their bases, a right-to-left run's
    /// visual order). Empty: the font's advances and kerning.
    pub placements: Vec<GlyphPlacement>,
}

/// A glow around overlay text ([`Text::glow`]): `color`'s alpha at the
/// glyph's edge, falling to nothing `radius` CSS pixels out. Its reach is
/// capped by the font's distance range: half `distance_range` atlas pixels
/// at the size drawn (a font cooked with `px_range 12` glows four times as
/// far as one with 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextGlow {
    /// Linear RGBA.
    pub color: [f32; 4],
    /// CSS pixels.
    pub radius: f32,
}

impl Text {
    /// White text of em size 1 at the origin on layer 0.
    pub fn new(font: FontHandle, text: impl Into<String>) -> Text {
        Text {
            font,
            text: text.into(),
            position: Vec3::ZERO,
            size: 1.0,
            rotation: 0.0,
            color: [1.0; 4],
            layer: 0,
            clip: None,
            tracking: 0.0,
            weight: 0.0,
            glow: None,
            placements: Vec::new(),
        }
    }
}

/// Sprites and text drawn over the finished frame in screen space: after
/// every post effect (exposure, bloom, tone mapping, TAA, grading), so they
/// are never tone mapped, exposed, jittered, blurred or depth tested. The
/// space is CSS pixels with the origin at the top left and y down: a
/// sprite's `position` is its anchor point, its `anchor` a fraction of its
/// size from the top-left corner, `rotation` clockwise on screen; a text's
/// `position` is the baseline origin of its first line, lines go down.
/// Colours are linear and alpha blended straight over the frame (an sRGB
/// output encodes them in hardware, any other output in the shader).
/// Sorted and batched as the world's sprites and text are: by (layer, kind,
/// texture or font), submission order within a run, the same textures and
/// MSDF fonts. Drawn on every tier; included in [`crate::Renderer::capture`]
/// and counted in [`crate::FrameStats`].
#[derive(Clone, Debug, PartialEq)]
pub struct Overlay {
    pub sprites: Vec<Sprite>,
    /// Overlay sprites drawn from a layer of a texture array instead of
    /// their texture ([`SpriteArray::sprite`] indexes [`Overlay::sprites`]):
    /// one draw per array whichever layer each shows, as in the world.
    pub sprite_arrays: Vec<SpriteArray>,
    pub texts: Vec<Text>,
    /// Glass panels: the frame under each, blurred ([`Glass`]).
    pub glass: Vec<Glass>,
    /// The overlay's size in CSS pixels; with `scale` (device pixels per CSS
    /// pixel), CSS (0, 0)-(width, height) maps onto the output's (0, 0)-
    /// (width·scale, height·scale). A `scale` of 0 fits the height to the
    /// output (scale = output height / `height`), whatever size the frame is
    /// rendered or captured at.
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    /// Clip rectangles `[x, y, width, height]` in overlay space, indexed by
    /// [`Sprite::clip`] and [`Text::clip`]; pixels outside are discarded.
    pub clips: Vec<[f32; 4]>,
}

/// The UI's layer among screen sprites and screen text (the IDL's
/// `UI_LAYER`): a game's overlay content at or below it draws under every
/// UI element, above it over every one (a HUD element of sprites on a UI
/// panel), in its layers' order. The overlay itself sorts by layer only;
/// og-runtime places the UI at [`UI_OVERLAY_LAYER`] and a game's screen
/// layers by [`screen_layer`].
pub const UI_LAYER: i32 = 1000;

/// The overlay layer the UI's painter level 0 draws on (level n on this plus
/// n): above every screen layer up to [`UI_LAYER`].
pub const UI_OVERLAY_LAYER: i32 = 1 << 20;

/// The overlay layer of a game's screen sprite or screen text of `layer`:
/// its own at or below [`UI_LAYER`], under the UI; above it, lifted past
/// every painter level the UI can use (2^20 of them), the order among
/// screen layers kept.
pub fn screen_layer(layer: i32) -> i32 {
    if layer > UI_LAYER {
        layer.saturating_add(1 << 21)
    } else {
        layer
    }
}

impl Overlay {
    /// An empty overlay of `width`×`height` CSS pixels at `scale` device
    /// pixels per CSS pixel.
    pub fn new(width: f32, height: f32, scale: f32) -> Overlay {
        Overlay {
            sprites: Vec::new(),
            sprite_arrays: Vec::new(),
            texts: Vec::new(),
            glass: Vec::new(),
            width,
            height,
            scale,
            clips: Vec::new(),
        }
    }

    /// Adds a clip rectangle `[x, y, width, height]` and returns its index.
    pub fn add_clip(&mut self, rect: [f32; 4]) -> u32 {
        self.clips.push(rect);
        (self.clips.len() - 1) as u32
    }
}

/// A glass panel of the overlay: the finished frame under it (the scene,
/// after every post effect; not other overlay content) blurred by `blur`
/// CSS px, tinted, and masked to a rounded rectangle. Drawn in the overlay's
/// order at `layer`, under what the layer draws over it. A frame with glass
/// renders its tonemap into a texture of its own, copies it to the output
/// and downsamples it into a six-level blur pyramid (src/glass.rs); a frame
/// without pays nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    /// The top left corner, CSS px.
    pub position: glam::Vec2,
    pub size: glam::Vec2,
    /// The corner radius, CSS px.
    pub radius: f32,
    /// The blur's radius, CSS px (0: the frame as it is, frosted only by the tint).
    pub blur: f32,
    /// Linear RGB, and in alpha how much of it covers the blur.
    pub tint: [f32; 4],
    /// The panel's opacity.
    pub alpha: f32,
    pub layer: i32,
    /// An index into [`Overlay::clips`].
    pub clip: Option<u32>,
}

static NEXT_SCENE_ID: AtomicU64 = AtomicU64::new(1);

/// Everything one frame draws. See the module documentation.
#[derive(Debug)]
pub struct RenderScene {
    id: u64,
    meshes: Vec<MeshData>,
    /// Per mesh: how many times [`RenderScene::set_mesh`] replaced it.
    mesh_revisions: Vec<u64>,
    /// Per mesh: [`RenderScene::set_vertex_encoding`].
    mesh_encodings: Vec<VertexEncoding>,
    textures: Vec<TextureSource>,
    /// Per texture: how many times [`RenderScene::set_texture`] replaced it.
    texture_revisions: Vec<u64>,
    /// What every material draws as.
    materials: Vec<StandardMaterial>,
    /// Per material: its shader and parameters.
    material_shaders: Vec<Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])>>,
    shaders: Vec<crate::MaterialShader>,
    /// The glTF-shaped form of the materials added as [`Material`].
    gltf_materials: Vec<Option<Material>>,
    material_revisions: Vec<u64>,
    fonts: Vec<MsdfFont>,
    /// Bumped by [`RenderScene::set_font`].
    font_revisions: Vec<u64>,
    /// Per mesh: its skin, for meshes added with `add_skinned_mesh`.
    skins: Vec<Option<SkinData>>,
    /// Per mesh: its blendshape targets.
    morphs: Vec<Option<MorphTargets>>,
    emitters: Vec<ParticleEmitter>,
    luts: Vec<ColorLut>,
    /// Per LUT slot, bumped when it is filled again or emptied.
    lut_revisions: Vec<u64>,
    palettes: Vec<crate::Palette>,
    /// Per palette, environment map and texture array: how many times its
    /// slot was refilled or emptied (the renderer uploads or frees it again).
    palette_revisions: Vec<u64>,
    environment_maps: Vec<crate::EnvironmentMapData>,
    environment_map_revisions: Vec<u64>,
    texture_arrays: Vec<crate::TextureArrayData>,
    texture_array_revisions: Vec<u64>,
    /// How each texture array is sampled by sprites (by array index).
    texture_array_filters: Vec<TextureFilter>,
    trail_styles: Vec<TrailStyle>,
    terrain_data: Vec<crate::terrain::TerrainSource>,
    /// Per terrain: how many times [`RenderScene::set_terrain`] replaced it.
    terrain_revisions: Vec<u64>,
    /// Objects to draw this frame, in any order.
    pub instances: Vec<Instance>,
    pub camera: Camera,
    pub light: DirectionalLight,
    /// Point and spot lights; the first [`MAX_LIGHTS`] are drawn.
    pub lights: Vec<Light>,
    pub environment: Environment,
    /// A cooked environment map lighting the scene instead of
    /// [`Environment::sky`] (three.js `scene.environment`), drawn as the
    /// background by [`Background::Sky`]; `None`: the sky lights it.
    pub environment_map: Option<crate::EnvironmentMap>,
    /// A hemisphere light, as three.js `HemisphereLight`: diffuse-only
    /// ambient irradiance `mix(ground, sky, 0.5 n.y + 0.5) * intensity`
    /// (sky, ground linear RGB; intensity 0: off). Unlike light baked into
    /// the environment map it adds no reflection.
    pub hemisphere: ([f32; 3], [f32; 3], f32),
    pub post: PostEffects,
    /// Stylisation after (and, for depth bands, with) the post stack: off
    /// by default ([`crate::Stylise`]).
    pub stylise: crate::Stylise,
    /// 2D quads drawn after the 3D scene, blended, sorted by layer.
    pub sprites: Vec<Sprite>,
    /// MSDF text, sorted with the sprites by layer.
    pub texts: Vec<Text>,
    pub debug: DebugView,
    /// The game's tick (drives beams' scroll and film grain; particle
    /// systems and trails carry their own).
    pub tick: u64,
    /// Ticks per second (the fixed step of particles and trails).
    pub tick_rate: f32,
    /// Running particle systems (state is kept by their id).
    pub particles: Vec<ParticleSystem>,
    /// What CPU-simulated particles with collision bounce off (the
    /// no-compute tier's stand-in for the depth buffer the compute path
    /// reads; [`crate::ParticleCollider`]). Empty by default.
    pub particle_colliders: Vec<crate::ParticleCollider>,
    pub trails: Vec<Trail>,
    pub ribbons: Vec<Ribbon>,
    pub beams: Vec<Beam>,
    /// Line sets drawn this frame (`lines.rs`): in the scene or over the
    /// finished frame (debug draw, screen lines).
    pub lines: Vec<crate::Lines>,
    /// Point clouds drawn this frame (`points.rs`).
    pub points: Vec<crate::Points>,
    /// The output's height in CSS pixels, which line widths and point sizes
    /// are measured in (0: one CSS pixel per device pixel).
    pub css_height: f32,
    /// Projected decals (the first [`crate::MAX_DECALS`] are drawn).
    pub decals: Vec<Decal>,
    /// The texture decals take their `uv_rect` from (`None`: solid colour).
    pub decal_atlas: Option<TextureHandle>,
    /// 2D lighting of the sprite layers (`None`: sprites are unlit).
    pub lighting_2d: Option<Lighting2d>,
    pub lights_2d: Vec<Light2d>,
    pub occluders_2d: Vec<Occluder2d>,
    /// Effects on single sprites (flash, glow, blend), by sprite index.
    pub sprite_fx: Vec<SpriteFx>,
    /// Textured triangles drawn in the sprite order (a 2D skeleton's
    /// attachments): [`crate::SpriteMesh`].
    pub sprite_meshes: Vec<crate::SpriteMesh>,
    /// Screen shake applied to the camera.
    pub shake: CameraShake,
    /// Reflection probes: boxes reflecting a cube captured inside them
    /// instead of the environment (the first [`crate::MAX_REFLECTION_PROBES`]).
    pub reflection_probes: Vec<crate::ReflectionProbe>,
    /// Each instance's transform in the previous frame (same order as
    /// `instances`), for per-object motion blur. Ignored unless its length
    /// equals `instances`'; then only the camera's motion blurs.
    pub previous_transforms: Vec<Mat4>,
    /// This frame's joint palettes ([`SkinnedInstance::palette`] indexes them).
    pub joint_palettes: Vec<JointPalette>,
    pub skinned_instances: Vec<SkinnedInstance>,
    /// Screen-space sprites and text over the finished frame (`None`: none).
    pub overlay: Option<Overlay>,
    /// Fog over the main pass (`None`: none).
    pub fog: Option<crate::Fog>,
    /// Fog lit as a participating medium (`volumetric.rs`; `None`: none).
    /// It replaces [`RenderScene::fog`] over the frame while it is set
    /// (and [`crate::RenderQuality::volumetric_fog`] lets it).
    pub volumetric_fog: Option<crate::VolumetricFog>,
    /// What the frame may cost: switches and sizes over what the rest of
    /// the scene asks for (`quality.rs`). The default draws all of it.
    pub quality: crate::RenderQuality,
    /// Baked lighting: lightmap pages, the lightmap uvs of the meshes drawn
    /// with them, light volumes (`lightmap.rs`). Empty by default.
    pub baked: crate::BakedLighting,
    /// Light cookies of point and spot lights, by index into
    /// [`RenderScene::lights`] (`area.rs`).
    pub light_cookies: Vec<crate::LightCookie>,
    /// Planar 3D water surfaces, drawn after the opaque scene and the sky.
    pub water: Vec<crate::Water>,
    /// Terrains drawn this frame (clipmaps of [`RenderScene::add_terrain`] data).
    pub terrains: Vec<crate::Terrain>,
    /// Copies scattered over this frame's terrains, placed and drawn by
    /// the renderer from cached chunks.
    pub scatters: Vec<crate::ScatterDraw>,
    /// Side-on 2D water bodies, drawn in the sprite order.
    pub water_2d: Vec<crate::Water2d>,
    /// 2D fluids drawn as metaballs in the sprite order (the first
    /// [`crate::MAX_FLUIDS`]; fluids of one group are one surface).
    pub metaballs: Vec<crate::Metaballs>,
    /// 3D particle fluids, drawn as one screen-space surface after the
    /// opaque scene, the sky and water.
    pub fluids_3d: Vec<crate::Fluid3d>,
    /// Sort keys of single sprites within their layer (top-down y sorting):
    /// a layer draws its sprites by ascending key, then texture, then
    /// submission order; a sprite without one has key 0.
    pub sprite_order: Vec<SpriteOrder>,
    /// Outlines of single world texts, by text index.
    pub text_outlines: Vec<TextOutline>,
    /// Tile layers: chunked quads drawn through the sprite batch, each
    /// under the sprites of its layer.
    pub tile_layers: Vec<TileLayer>,
    /// Trails drawn in the sprite order at a layer (2D) instead of with the
    /// 3D scene: `(Trail::id, layer)`.
    pub trail_layers: Vec<(u64, i32)>,
    /// Sprites drawn from a layer of a texture array instead of their
    /// texture, by sprite index (world sprites only; the last entry for an
    /// index wins). Sprites of one array draw as one run whatever their
    /// layers, so a sheet of frames in an array batches across frames.
    pub sprite_arrays: Vec<SpriteArray>,
    /// Drop shadows of single world sprites, by sprite index (the last
    /// entry for an index wins): each a flattened dark copy of the sprite
    /// on the ground plane y = 0 cast along [`RenderScene::light`], drawn
    /// before every sprite of its layer.
    pub sprite_shadows: Vec<SpriteShadow>,
    /// Three.js layer masks of the instances, by index in `instances`
    /// (bit `n` set: on layer `n`): an index past the end is on layer 0
    /// alone (mask 1), as every object is in three.js. A camera draws the
    /// instances whose mask shares a bit with its own
    /// ([`RenderScene::camera_layers`], [`crate::View::layers`]); the shadow
    /// maps take every caster whatever its layers, so a body hidden from a
    /// first-person camera still casts its shadow.
    pub instance_layers: Vec<u32>,
    /// The same for `skinned_instances`.
    pub skinned_layers: Vec<u32>,
    /// The same for the world `sprites` (with their drop shadows, glow and
    /// afterimages drawn from them).
    pub sprite_layers: Vec<u32>,
    /// How each world sprite faces the camera, by index in `sprites`
    /// (billboards): an index past the end is [`SpriteFacing::None`], the
    /// xy plane. A faced sprite without a [`RenderScene::sprite_order`] key
    /// (and tone mapped) draws among the transparent meshes, back to front
    /// with them by the view depth of its anchor from the eye that draws
    /// it, its drop shadow before them; with a key it keeps the sprites'
    /// (layer, key) order. Culled by its bounds (a sphere round the anchor
    /// through the farthest corner) against that camera's frustum. Its
    /// drop shadow is cast by the quad stood upright facing the sun, so it
    /// shows the sprite's whole shape from every camera.
    pub sprite_facing: Vec<SpriteFacing>,
    /// World texts turned to the camera ([`TextFacing`], by text index; the
    /// last entry for an index wins; a text without one stays in the xy
    /// plane), as [`RenderScene::sprite_facing`] turns sprites.
    pub text_facing: Vec<TextFacing>,
    /// The same for `lines` and `points` (scene sets; overlay and screen
    /// sets draw on the main frame whatever their masks).
    pub line_layers: Vec<u32>,
    pub point_layers: Vec<u32>,
    /// The same for `particles` (billboards and 2D sprite particles alike).
    pub particle_layers: Vec<u32>,
    /// The main camera's layer mask (three.js `Camera.layers`): it draws
    /// the instances, skinned instances, world sprites, scene lines and
    /// points and particle systems whose masks share a bit with it, as each
    /// [`crate::View`] does with its own. 1 by default; 0 draws none of
    /// them. What carries no mask (the sky, terrain, scattered copies,
    /// water, trails, text, tile layers and sprite meshes) draws on every
    /// camera.
    pub camera_layers: u32,
    /// Secondary cameras drawn into render targets or viewports of the
    /// frame, before it in [`crate::View::order`] (`views.rs`).
    pub views: Vec<crate::View>,
    /// Custom post passes ([`RenderScene::add_post`], `post_custom.rs`) and
    /// their parameters, by handle.
    pub posts: Vec<crate::PostShader>,
    pub post_params: Vec<[[f32; 4]; crate::MAX_SHADER_PARAMS]>,
    /// The custom post passes the frame runs, in order (at most
    /// [`crate::MAX_CUSTOM_POSTS`]), after the built-in effects and before
    /// exposure, bloom and the tone curve (`PostEffects.custom` in the API).
    pub custom_post: Vec<crate::PostHandle>,
    /// How each texture is sampled by sprites, tiles and text (by texture index).
    texture_filters: Vec<TextureFilter>,
    /// Live slots and generations of the removable kinds.
    mesh_slots: Slots,
    texture_slots: Slots,
    material_slots: Slots,
    font_slots: Slots,
    environment_map_slots: Slots,
    texture_array_slots: Slots,
    palette_slots: Slots,
    lut_slots: Slots,
}

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderScene {
    pub fn new() -> Self {
        RenderScene {
            id: NEXT_SCENE_ID.fetch_add(1, Ordering::Relaxed),
            meshes: Vec::new(),
            mesh_revisions: Vec::new(),
            mesh_encodings: Vec::new(),
            textures: Vec::new(),
            texture_revisions: Vec::new(),
            materials: Vec::new(),
            material_shaders: Vec::new(),
            shaders: Vec::new(),
            gltf_materials: Vec::new(),
            material_revisions: Vec::new(),
            fonts: Vec::new(),
            font_revisions: Vec::new(),
            instances: Vec::new(),
            camera: Camera::default(),
            hemisphere: ([0.0; 3], [0.0; 3], 0.0),
            light: DirectionalLight::default(),
            lights: Vec::new(),
            environment: Environment::default(),
            environment_map: None,
            post: PostEffects::default(),
            stylise: crate::Stylise::default(),
            sprites: Vec::new(),
            texts: Vec::new(),
            debug: DebugView::None,
            skins: Vec::new(),
            morphs: Vec::new(),
            emitters: Vec::new(),
            luts: Vec::new(),
            lut_revisions: Vec::new(),
            palettes: Vec::new(),
            palette_revisions: Vec::new(),
            environment_maps: Vec::new(),
            environment_map_revisions: Vec::new(),
            texture_arrays: Vec::new(),
            texture_array_revisions: Vec::new(),
            texture_array_filters: Vec::new(),
            trail_styles: Vec::new(),
            terrain_data: Vec::new(),
            terrain_revisions: Vec::new(),
            instance_layers: Vec::new(),
            skinned_layers: Vec::new(),
            sprite_layers: Vec::new(),
            sprite_facing: Vec::new(),
            text_facing: Vec::new(),
            line_layers: Vec::new(),
            point_layers: Vec::new(),
            particle_layers: Vec::new(),
            camera_layers: 1,
            posts: Vec::new(),
            post_params: Vec::new(),
            custom_post: Vec::new(),
            views: Vec::new(),
            tick: 0,
            tick_rate: 60.0,
            particles: Vec::new(),
            particle_colliders: Vec::new(),
            trails: Vec::new(),
            ribbons: Vec::new(),
            lines: Vec::new(),
            points: Vec::new(),
            css_height: 0.0,
            beams: Vec::new(),
            decals: Vec::new(),
            decal_atlas: None,
            lighting_2d: None,
            lights_2d: Vec::new(),
            occluders_2d: Vec::new(),
            sprite_fx: Vec::new(),
            sprite_meshes: Vec::new(),
            reflection_probes: Vec::new(),
            shake: CameraShake::default(),
            previous_transforms: Vec::new(),
            joint_palettes: Vec::new(),
            skinned_instances: Vec::new(),
            overlay: None,
            fog: None,
            volumetric_fog: None,
            quality: crate::RenderQuality::default(),
            baked: crate::BakedLighting::default(),
            light_cookies: Vec::new(),
            water: Vec::new(),
            terrains: Vec::new(),
            scatters: Vec::new(),
            water_2d: Vec::new(),
            metaballs: Vec::new(),
            fluids_3d: Vec::new(),
            sprite_order: Vec::new(),
            text_outlines: Vec::new(),
            tile_layers: Vec::new(),
            trail_layers: Vec::new(),
            sprite_arrays: Vec::new(),
            sprite_shadows: Vec::new(),
            texture_filters: Vec::new(),
            mesh_slots: Slots::default(),
            texture_slots: Slots::default(),
            material_slots: Slots::default(),
            font_slots: Slots::default(),
            environment_map_slots: Slots::default(),
            texture_array_slots: Slots::default(),
            palette_slots: Slots::default(),
            lut_slots: Slots::default(),
        }
    }

    /// Samples a texture of this scene nearest (pixel art) or linear (the
    /// default) wherever sprites, tiles and particles draw it. `false` for
    /// a handle of another scene.
    pub fn set_texture_filter(&mut self, texture: TextureHandle, filter: TextureFilter) -> bool {
        let i = texture.index();
        if !self.texture_slots.live(i, texture.generation()) {
            return false;
        }
        if self.texture_filters.len() <= i {
            self.texture_filters.resize(i + 1, TextureFilter::Linear);
        }
        self.texture_filters[i] = filter;
        true
    }

    /// How sprites sample `texture`.
    pub fn texture_filter(&self, texture: TextureHandle) -> TextureFilter {
        self.texture_filters
            .get(texture.index())
            .copied()
            .unwrap_or_default()
    }

    /// Adds a mesh with a per-vertex skin (four joints and weights each),
    /// drawn posed by [`RenderScene::skinned_instances`]. The skin must have
    /// one entry per vertex ([`crate::RenderError::InvalidData`] at upload).
    pub fn add_skinned_mesh(&mut self, mesh: MeshData, skin: SkinData) -> MeshHandle {
        // A new slot: the renderer sets a skin up once, for a new mesh.
        let h = self.push_mesh(mesh, false);
        self.skins[h.index()] = Some(skin);
        h
    }

    /// Gives a mesh blendshape targets (one delta per vertex per target, at
    /// most [`crate::MAX_MORPH_TARGETS`]); draw it with
    /// [`RenderScene::skinned_instances`] (weights per instance; a mesh
    /// without a skin stays in its bind pose). Checked at upload. Set them
    /// before the mesh's first frame; `false` for a handle of another scene.
    pub fn set_morph_targets(&mut self, mesh: MeshHandle, targets: MorphTargets) -> bool {
        if !self.mesh_slots.live(mesh.index(), mesh.generation()) {
            return false;
        }
        match self.morphs.get_mut(mesh.index()) {
            Some(m) => {
                *m = Some(targets);
                true
            }
            None => false,
        }
    }

    pub(crate) fn morphs(&self) -> &[Option<MorphTargets>] {
        &self.morphs
    }

    pub fn skin(&self, handle: MeshHandle) -> Option<&SkinData> {
        self.skins.get(handle.index())?.as_ref()
    }

    /// Adds terrain data (heights and a control map, an asset); draw it
    /// with a [`crate::Terrain`] in [`RenderScene::terrains`]. Checked at
    /// its first frame ([`crate::RenderError::InvalidData`]).
    pub fn add_terrain(&mut self, data: crate::TerrainData) -> TerrainHandle {
        self.terrain_data
            .push(crate::terrain::TerrainSource::Data(data));
        self.terrain_revisions.push(0);
        TerrainHandle(self.terrain_data.len() as u32 - 1)
    }

    /// Adds a streamed terrain: its tiles arrive one by one with
    /// [`RenderScene::set_terrain_tile`] (the top level's before it draws),
    /// `range` the lowest and highest height. A tile that has not arrived
    /// shows its parent's.
    pub fn add_terrain_stream(
        &mut self,
        layout: crate::TerrainLayout,
        range: (f32, f32),
    ) -> TerrainHandle {
        self.terrain_data
            .push(crate::terrain::TerrainSource::Stream {
                layout,
                range,
                tiles: Default::default(),
            });
        self.terrain_revisions.push(0);
        TerrainHandle(self.terrain_data.len() as u32 - 1)
    }

    /// Makes a terrain streamed again from the start (no tile resident),
    /// as after [`RenderScene::add_terrain_stream`]. `false` for a handle
    /// of another scene.
    pub fn set_terrain_stream(
        &mut self,
        handle: TerrainHandle,
        layout: crate::TerrainLayout,
        range: (f32, f32),
    ) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(d) => {
                *d = crate::terrain::TerrainSource::Stream {
                    layout,
                    range,
                    tiles: Default::default(),
                };
                self.terrain_revisions[handle.index()] += 1;
                true
            }
            None => false,
        }
    }

    /// Hands a streamed terrain one of its tiles (`None`: drops it). The
    /// renderer draws what it holds of the tiles it wants. `false` for a
    /// handle of another scene, a terrain in memory, a tile index beyond
    /// its layout or a tile whose heights, normals or control texels are not
    /// the lengths its layout uploads (the tile is not taken).
    pub fn set_terrain_tile(
        &mut self,
        handle: TerrainHandle,
        index: u32,
        tile: Option<std::sync::Arc<crate::TerrainTile>>,
    ) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(crate::terrain::TerrainSource::Stream { layout, tiles, .. })
                if index < layout.tile_count() =>
            {
                match tile {
                    Some(t) if !t.fits(layout) => return false,
                    Some(t) => {
                        tiles.insert(index, t);
                    }
                    None => {
                        tiles.remove(&index);
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// The tiles a streamed terrain holds now (`None` for a terrain in
    /// memory).
    pub fn terrain_tiles(
        &self,
        handle: TerrainHandle,
    ) -> Option<&og_core::IdMap<u32, std::sync::Arc<crate::TerrainTile>>> {
        match self.terrain_data.get(handle.index())? {
            crate::terrain::TerrainSource::Stream { tiles, .. } => Some(tiles),
            crate::terrain::TerrainSource::Data(_) => None,
        }
    }

    /// The layout a terrain is drawn with (a field in memory's is chosen
    /// by the renderer: tiles of [`crate::TERRAIN_TILE`] quads, or larger).
    pub fn terrain_layout(&self, handle: TerrainHandle) -> Option<crate::TerrainLayout> {
        self.terrain_data.get(handle.index())?.layout().ok()
    }

    /// Replaces a terrain's data (a runtime edit of its heights or its
    /// control map); the renderer uploads it again before the next frame
    /// that draws it. `false` for a handle of another scene.
    pub fn set_terrain(&mut self, handle: TerrainHandle, data: crate::TerrainData) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(d) => {
                *d = crate::terrain::TerrainSource::Data(data);
                self.terrain_revisions[handle.index()] += 1;
                true
            }
            None => false,
        }
    }

    /// A terrain's field in memory (`None` for a streamed one).
    pub fn terrain(&self, handle: TerrainHandle) -> Option<&crate::TerrainData> {
        match self.terrain_data.get(handle.index())? {
            crate::terrain::TerrainSource::Data(d) => Some(d),
            crate::terrain::TerrainSource::Stream { .. } => None,
        }
    }

    pub(crate) fn terrain_sources(&self) -> (&[crate::terrain::TerrainSource], &[u64]) {
        (&self.terrain_data, &self.terrain_revisions)
    }

    /// How far from the camera terrain `index` (of [`RenderScene::terrains`])
    /// needs its finest tiles for the copies scattered over it.
    pub fn scatter_detail(&self, index: usize) -> f32 {
        self.scatters
            .iter()
            .filter(|s| s.terrain == index)
            .map(|s| s.scatter.distance)
            .fold(0.0, f32::max)
    }

    /// Adds a particle emitter description (an asset).
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) -> EmitterHandle {
        self.emitters.push(emitter);
        EmitterHandle(self.emitters.len() as u32 - 1)
    }

    /// Replaces an emitter (running systems keep their particles while the
    /// capacity is unchanged). `false` for a handle of another scene.
    pub fn set_emitter(&mut self, handle: EmitterHandle, emitter: ParticleEmitter) -> bool {
        match self.emitters.get_mut(handle.index()) {
            Some(e) => {
                *e = emitter;
                true
            }
            None => false,
        }
    }

    pub fn emitter(&self, handle: EmitterHandle) -> Option<&ParticleEmitter> {
        self.emitters.get(handle.index())
    }

    /// Adds a 32³ colour grading table ([`ColorLut`]). A slot freed by
    /// [`RenderScene::remove_lut`] is reused under a new generation.
    pub fn add_lut(&mut self, lut: ColorLut) -> LutHandle {
        let (i, generation, reused) = self.lut_slots.take(self.luts.len(), true);
        if reused {
            self.luts[i] = lut;
            self.lut_revisions[i] += 1;
        } else {
            self.luts.push(lut);
            self.lut_revisions.push(0);
        }
        LutHandle::from_parts(i, generation)
    }

    /// Removes a colour LUT: the renderer frees its 3D texture before the
    /// next frame and a later add reuses the slot under a new generation.
    /// A frame whose colour grading still names it is refused
    /// ([`crate::RenderError::InvalidHandle`]). False for a handle that is
    /// not live.
    pub fn remove_lut(&mut self, handle: LutHandle) -> bool {
        let i = handle.index();
        if !self.lut_slots.live(i, handle.generation()) {
            return false;
        }
        self.luts[i] = ColorLut {
            size: 0,
            rgba: Vec::new(),
        };
        self.lut_revisions[i] += 1;
        self.lut_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a LUT of this scene that was not removed.
    pub fn contains_lut(&self, handle: LutHandle) -> bool {
        self.lut_slots.live(handle.index(), handle.generation())
    }

    /// A live LUT.
    pub fn lut(&self, handle: LutHandle) -> Option<&ColorLut> {
        self.contains_lut(handle)
            .then(|| &self.luts[handle.index()])
    }

    pub(crate) fn lut_revisions(&self) -> &[u64] {
        &self.lut_revisions
    }

    /// Whether slot `index` holds a live LUT.
    pub(crate) fn lut_live(&self, index: usize) -> bool {
        let g = self.lut_slots.generation(index);
        self.lut_slots.live(index, g)
    }

    /// Adds a prefiltered environment map (an asset; og-cook's `type
    /// environment`); [`RenderScene::environment_map`] picks the one a
    /// frame is lit by. A slot freed by [`RenderScene::remove_environment_map`]
    /// is reused under a new generation.
    pub fn add_environment_map(&mut self, map: crate::EnvironmentMapData) -> EnvironmentMapHandle {
        let (i, generation, reused) = self
            .environment_map_slots
            .take(self.environment_maps.len(), true);
        if reused {
            self.environment_maps[i] = map;
            self.environment_map_revisions[i] += 1;
        } else {
            self.environment_maps.push(map);
            self.environment_map_revisions.push(0);
        }
        EnvironmentMapHandle::from_parts(i, generation)
    }

    /// Removes an environment map: the renderer frees its cube before the
    /// next frame and a later add reuses the slot under a new generation.
    /// A frame whose [`RenderScene::environment_map`] still names it is
    /// refused ([`crate::RenderError::InvalidHandle`]): light the scene
    /// with another map or the sky first. False for a handle that is not
    /// live.
    pub fn remove_environment_map(&mut self, handle: EnvironmentMapHandle) -> bool {
        let i = handle.index();
        if !self.environment_map_slots.live(i, handle.generation()) {
            return false;
        }
        self.environment_maps[i] = crate::EnvironmentMapData {
            size: 0,
            texels: Vec::new(),
            irradiance: [[0.0; 3]; 9],
        };
        self.environment_map_revisions[i] += 1;
        self.environment_map_slots.remove(i, true);
        true
    }

    /// Whether `handle` names an environment map of this scene that was
    /// not removed.
    pub fn contains_environment_map(&self, handle: EnvironmentMapHandle) -> bool {
        self.environment_map_slots
            .live(handle.index(), handle.generation())
    }

    /// A live environment map's data.
    pub fn environment_map_data(
        &self,
        handle: EnvironmentMapHandle,
    ) -> Option<&crate::EnvironmentMapData> {
        self.contains_environment_map(handle)
            .then(|| &self.environment_maps[handle.index()])
    }

    pub(crate) fn environment_maps(&self) -> &[crate::EnvironmentMapData] {
        &self.environment_maps
    }

    pub(crate) fn environment_map_revisions(&self) -> &[u64] {
        &self.environment_map_revisions
    }

    /// Whether slot `index` holds a live environment map, and its generation.
    pub(crate) fn environment_map_slot(&self, index: usize) -> (bool, u8) {
        let g = self.environment_map_slots.generation(index);
        (self.environment_map_slots.live(index, g), g)
    }

    /// Adds a texture array (an asset; og-cook's `type texture-array`):
    /// layers of one size, encoding and mip count, uploaded as one
    /// `D2Array` texture. Refused when the layers disagree. A slot freed
    /// by [`RenderScene::remove_texture_array`] is reused under a new
    /// generation.
    pub fn add_texture_array(
        &mut self,
        data: crate::TextureArrayData,
    ) -> Result<crate::TextureArrayHandle, String> {
        data.info()?;
        let (i, generation, reused) = self
            .texture_array_slots
            .take(self.texture_arrays.len(), true);
        if reused {
            self.texture_arrays[i] = data;
            self.texture_array_revisions[i] += 1;
            if let Some(f) = self.texture_array_filters.get_mut(i) {
                *f = TextureFilter::Linear;
            }
        } else {
            self.texture_arrays.push(data);
            self.texture_array_revisions.push(0);
        }
        Ok(crate::TextureArrayHandle::from_parts(i, generation))
    }

    /// Removes a texture array: the renderer frees it before the next frame
    /// and a later add reuses the slot under a new generation. A sprite
    /// still drawn from it ([`RenderScene::sprite_arrays`]) is refused at
    /// the next frame ([`crate::RenderError::InvalidHandle`]); a material
    /// sampling it forgets it (its array maps read as none), as a removed
    /// texture's. False for a handle that is not live.
    pub fn remove_texture_array(&mut self, handle: crate::TextureArrayHandle) -> bool {
        let i = handle.index();
        if !self.texture_array_slots.live(i, handle.generation()) {
            return false;
        }
        self.texture_arrays[i] = crate::TextureArrayData { layers: Vec::new() };
        self.texture_array_revisions[i] += 1;
        if let Some(f) = self.texture_array_filters.get_mut(i) {
            *f = TextureFilter::Linear;
        }
        self.texture_array_slots.remove(i, true);
        // Materials sampling it draw without it from now on.
        let slot = handle.slot();
        for (m, revision) in self
            .materials
            .iter_mut()
            .zip(self.material_revisions.iter_mut())
        {
            if m.forget_texture_array(slot) {
                *revision += 1;
            }
        }
        true
    }

    /// Whether `handle` names a texture array of this scene that was not
    /// removed.
    pub fn contains_texture_array(&self, handle: crate::TextureArrayHandle) -> bool {
        self.texture_array_slots
            .live(handle.index(), handle.generation())
    }

    /// A live texture array's layers.
    pub fn texture_array(
        &self,
        handle: crate::TextureArrayHandle,
    ) -> Option<&crate::TextureArrayData> {
        self.contains_texture_array(handle)
            .then(|| &self.texture_arrays[handle.index()])
    }

    /// Samples a texture array nearest (pixel art) or linear (the default)
    /// wherever sprites draw it. False for a handle that is not live.
    pub fn set_texture_array_filter(
        &mut self,
        handle: crate::TextureArrayHandle,
        filter: TextureFilter,
    ) -> bool {
        let i = handle.index();
        if !self.texture_array_slots.live(i, handle.generation()) {
            return false;
        }
        if self.texture_array_filters.len() <= i {
            self.texture_array_filters
                .resize(i + 1, TextureFilter::Linear);
        }
        self.texture_array_filters[i] = filter;
        true
    }

    /// How sprites sample a texture array.
    pub fn texture_array_filter(&self, handle: crate::TextureArrayHandle) -> TextureFilter {
        self.texture_array_filters
            .get(handle.index())
            .copied()
            .unwrap_or_default()
    }

    /// Every slot's layers, by handle index (a removed slot holds none).
    pub fn texture_arrays(&self) -> &[crate::TextureArrayData] {
        &self.texture_arrays
    }

    pub(crate) fn texture_array_revisions(&self) -> &[u64] {
        &self.texture_array_revisions
    }

    /// Whether slot `index` holds a live texture array, and its generation.
    pub(crate) fn texture_array_slot(&self, index: usize) -> (bool, u8) {
        let g = self.texture_array_slots.generation(index);
        (self.texture_array_slots.live(index, g), g)
    }

    /// Adds a trail style (an asset).
    pub fn add_trail_style(&mut self, style: TrailStyle) -> TrailStyleHandle {
        self.trail_styles.push(style);
        TrailStyleHandle(self.trail_styles.len() as u32 - 1)
    }

    pub(crate) fn emitters(&self) -> &[ParticleEmitter] {
        &self.emitters
    }

    pub(crate) fn luts(&self) -> &[ColorLut] {
        &self.luts
    }

    /// Adds a colour palette (an asset; og-cook's `.hex` and `.gpl`) for
    /// [`crate::Stylise`]'s quantisation; at most
    /// [`crate::MAX_PALETTE_COLORS`] colours are kept.
    pub fn add_palette(&mut self, mut palette: crate::Palette) -> crate::PaletteHandle {
        palette.colors.truncate(crate::MAX_PALETTE_COLORS);
        let (i, generation, reused) = self.palette_slots.take(self.palettes.len(), true);
        if reused {
            self.palettes[i] = palette;
            self.palette_revisions[i] += 1;
        } else {
            self.palettes.push(palette);
            self.palette_revisions.push(0);
        }
        crate::PaletteHandle::from_parts(i, generation)
    }

    /// Removes a palette: the renderer frees its texture before the next
    /// frame and a later add reuses the slot under a new generation. A
    /// frame whose [`RenderScene::stylise`] still quantises to it is
    /// refused ([`crate::RenderError::InvalidHandle`]). False for a handle
    /// that is not live.
    pub fn remove_palette(&mut self, handle: crate::PaletteHandle) -> bool {
        let i = handle.index();
        if !self.palette_slots.live(i, handle.generation()) {
            return false;
        }
        self.palettes[i] = crate::Palette::default();
        self.palette_revisions[i] += 1;
        self.palette_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a palette of this scene that was not removed.
    pub fn contains_palette(&self, handle: crate::PaletteHandle) -> bool {
        self.palette_slots.live(handle.index(), handle.generation())
    }

    /// A live palette.
    pub fn palette(&self, handle: crate::PaletteHandle) -> Option<&crate::Palette> {
        self.contains_palette(handle)
            .then(|| &self.palettes[handle.index()])
    }

    /// Every slot's palette, by handle index (a removed slot holds no
    /// colours).
    pub fn palettes(&self) -> &[crate::Palette] {
        &self.palettes
    }

    pub(crate) fn palette_revisions(&self) -> &[u64] {
        &self.palette_revisions
    }

    /// Whether slot `index` holds a live palette.
    pub(crate) fn palette_live(&self, index: usize) -> bool {
        let g = self.palette_slots.generation(index);
        self.palette_slots.live(index, g)
    }

    pub(crate) fn trail_styles(&self) -> &[TrailStyle] {
        &self.trail_styles
    }

    pub(crate) fn skins(&self) -> &[Option<SkinData>] {
        &self.skins
    }

    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshHandle {
        self.push_mesh(mesh, true)
    }

    fn push_mesh(&mut self, mesh: MeshData, reuse: bool) -> MeshHandle {
        let (i, generation, reused) = self.mesh_slots.take(self.meshes.len(), reuse);
        if reused {
            self.meshes[i] = mesh;
            self.mesh_revisions[i] += 1;
            self.mesh_encodings[i] = VertexEncoding::Full;
        } else {
            self.meshes.push(mesh);
            self.skins.push(None);
            self.morphs.push(None);
            self.mesh_revisions.push(0);
            self.mesh_encodings.push(VertexEncoding::Full);
        }
        MeshHandle::from_parts(i, generation)
    }

    /// Removes a mesh: the renderer frees its buffers before the next frame
    /// and a later [`RenderScene::add_mesh`] reuses the slot under a new
    /// generation, so this handle (and every copy of it) is refused from
    /// now on: by `set_mesh` and the other setters, and as an instance's
    /// mesh ([`crate::RenderError::InvalidHandle`]). A skinned or morphed
    /// mesh's slot is not reused (its skin and blendshape data stay with
    /// the renderer). False for a handle that is not live.
    pub fn remove_mesh(&mut self, handle: MeshHandle) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation()) {
            return false;
        }
        let reuse = self.skins[i].is_none() && self.morphs[i].is_none();
        self.meshes[i] = MeshData::default();
        self.mesh_revisions[i] += 1;
        self.mesh_slots.remove(i, reuse);
        true
    }

    /// Whether `handle` names a mesh of this scene that was not removed.
    pub fn contains_mesh(&self, handle: MeshHandle) -> bool {
        self.mesh_slots.live(handle.index(), handle.generation())
    }

    /// Replaces a mesh's vertices and indices (cloth, generated geometry);
    /// the renderer uploads it again before the next frame, in place when
    /// the sizes match. False for a bad handle or a skinned or morphed mesh.
    pub fn set_mesh(&mut self, handle: MeshHandle, mesh: MeshData) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation())
            || self.skins[i].is_some()
            || self.morphs[i].is_some()
        {
            return false;
        }
        self.meshes[i] = mesh;
        self.mesh_revisions[i] += 1;
        true
    }

    pub(crate) fn mesh_revisions(&self) -> &[u64] {
        &self.mesh_revisions
    }

    /// Sets how a mesh's vertices are held on the GPU (default
    /// [`VertexEncoding::Full`]); the renderer uploads it again before the
    /// next frame when this changes it. False for a bad handle.
    pub fn set_vertex_encoding(&mut self, handle: MeshHandle, encoding: VertexEncoding) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation()) {
            return false;
        }
        if self.mesh_encodings[i] != encoding {
            self.mesh_encodings[i] = encoding;
            self.mesh_revisions[i] += 1;
        }
        true
    }

    /// A mesh's [`VertexEncoding`] (`None`: a bad handle).
    pub fn vertex_encoding(&self, handle: MeshHandle) -> Option<VertexEncoding> {
        self.contains_mesh(handle)
            .then(|| self.mesh_encodings[handle.index()])
    }

    pub(crate) fn mesh_encodings(&self) -> &[VertexEncoding] {
        &self.mesh_encodings
    }

    /// Adds an RGBA8 texture (mips generated on the GPU when asked).
    pub fn add_texture(&mut self, texture: TextureData) -> TextureHandle {
        self.add_texture_source(TextureSource::Rgba(texture))
    }

    /// Replaces a texture (a streamed texture over its placeholder, or back):
    /// the renderer uploads it again before the next frame, and every
    /// material, sprite, tile, trail, font, decal and terrain layer that
    /// samples it draws the new one. False for a bad handle.
    pub fn set_texture(&mut self, handle: TextureHandle, texture: TextureData) -> bool {
        self.replace_texture(handle, TextureSource::Rgba(texture))
    }

    /// [`RenderScene::set_texture`] with mip levels already encoded.
    pub fn set_texture_upload(&mut self, handle: TextureHandle, upload: TextureUpload) -> bool {
        self.replace_texture(handle, TextureSource::Encoded(upload))
    }

    fn replace_texture(&mut self, handle: TextureHandle, source: TextureSource) -> bool {
        let i = handle.index();
        if !self.texture_slots.live(i, handle.generation()) {
            return false;
        }
        self.textures[i] = source;
        self.texture_revisions[i] += 1;
        true
    }

    pub(crate) fn texture_revisions(&self) -> &[u64] {
        &self.texture_revisions
    }

    /// Adds a texture whose mip levels are already encoded, in one of
    /// [`crate::RenderInfo::texture_formats`]. Checked (and refused with
    /// [`crate::RenderError::InvalidData`] or `Unsupported`) when uploaded.
    pub fn add_texture_upload(&mut self, upload: TextureUpload) -> TextureHandle {
        self.add_texture_source(TextureSource::Encoded(upload))
    }

    /// Adds a texture of either kind ([`TextureSource`]).
    pub fn add_texture_source(&mut self, source: TextureSource) -> TextureHandle {
        let (i, generation, reused) = self.texture_slots.take(self.textures.len(), true);
        if reused {
            self.textures[i] = source;
            self.texture_revisions[i] += 1;
            if let Some(f) = self.texture_filters.get_mut(i) {
                *f = TextureFilter::Linear;
            }
        } else {
            self.textures.push(source);
            self.texture_revisions.push(0);
        }
        TextureHandle::from_parts(i, generation)
    }

    /// Removes a texture: the renderer frees it before the next frame and a
    /// later add reuses the slot under a new generation. Every material
    /// slot that sampled it is cleared (those materials draw without it from
    /// the next frame), and so is the decal atlas; a sprite or a font
    /// atlas still naming it is refused at the next frame
    /// ([`crate::RenderError::InvalidHandle`]): remove the font first.
    /// False for a handle that is not live.
    pub fn remove_texture(&mut self, handle: TextureHandle) -> bool {
        let i = handle.index();
        if !self.texture_slots.live(i, handle.generation()) {
            return false;
        }
        self.textures[i] = TextureSource::Rgba(TextureData {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        self.texture_revisions[i] += 1;
        if let Some(f) = self.texture_filters.get_mut(i) {
            *f = TextureFilter::Linear;
        }
        let slot = handle.slot();
        for (m, revision) in self
            .materials
            .iter_mut()
            .zip(self.material_revisions.iter_mut())
        {
            if m.forget_texture(slot) {
                *revision += 1;
            }
        }
        for m in self.gltf_materials.iter_mut().flatten() {
            for t in [
                &mut m.base_color_texture,
                &mut m.metallic_roughness_texture,
                &mut m.normal_texture,
                &mut m.emissive_texture,
            ] {
                if *t == Some(handle) {
                    *t = None;
                }
            }
        }
        if self.decal_atlas == Some(handle) {
            self.decal_atlas = None;
        }
        self.texture_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a texture of this scene that was not removed.
    pub fn contains_texture(&self, handle: TextureHandle) -> bool {
        self.texture_slots.live(handle.index(), handle.generation())
    }

    /// Adds a glTF-shaped material (drawn as its [`StandardMaterial`] form).
    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        let standard = StandardMaterial::from(&material);
        self.push_material(standard, Some(material))
    }

    /// Adds a three.js-style standard material.
    pub fn add_standard_material(&mut self, material: StandardMaterial) -> MaterialHandle {
        self.push_material(material, None)
    }

    fn push_material(
        &mut self,
        material: StandardMaterial,
        gltf: Option<Material>,
    ) -> MaterialHandle {
        let (i, generation, reused) = self.material_slots.take(self.materials.len(), true);
        if reused {
            self.materials[i] = material;
            self.gltf_materials[i] = gltf;
            self.material_revisions[i] += 1;
        } else {
            self.materials.push(material);
            self.gltf_materials.push(gltf);
            self.material_revisions.push(0);
        }
        MaterialHandle::from_parts(i, generation)
    }

    /// Removes a material: the renderer frees its bind groups before the
    /// next frame and a later add reuses the slot under a new generation;
    /// an instance still drawing with this handle is refused
    /// ([`crate::RenderError::InvalidHandle`]). Its material shader and
    /// parameters go with it. False for a handle that is not live.
    pub fn remove_material(&mut self, handle: MaterialHandle) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = StandardMaterial::default();
        self.gltf_materials[i] = None;
        if let Some(s) = self.material_shaders.get_mut(i) {
            *s = None;
        }
        self.material_revisions[i] += 1;
        self.material_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a material of this scene that was not removed.
    pub fn contains_material(&self, handle: MaterialHandle) -> bool {
        self.material_slots
            .live(handle.index(), handle.generation())
    }

    /// Replaces a material with a glTF-shaped one; the renderer re-uploads it
    /// on the next frame. Returns `false` (and changes nothing) for a handle
    /// of another scene.
    pub fn set_material(&mut self, handle: MaterialHandle, material: Material) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = StandardMaterial::from(&material);
        self.gltf_materials[i] = Some(material);
        self.material_revisions[i] += 1;
        true
    }

    /// Replaces a material with a standard one (a material edit: the next
    /// frame draws it). Returns `false` for a handle of another scene.
    pub fn set_standard_material(
        &mut self,
        handle: MaterialHandle,
        material: StandardMaterial,
    ) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = material;
        self.gltf_materials[i] = None;
        self.material_revisions[i] += 1;
        true
    }

    /// Adds one instance (the same as pushing onto [`RenderScene::instances`]).
    pub fn add_instance(&mut self, mesh: MeshHandle, material: MaterialHandle, transform: Mat4) {
        self.instances.push(Instance {
            mesh,
            material,
            transform,
            material_layer: None,
            fx: InstanceFx::NONE,
            cast_shadows: true,
            receive_shadows: true,
        });
    }

    /// Adds an MSDF font whose atlas is a texture of this scene.
    pub fn add_font(&mut self, font: MsdfFont) -> FontHandle {
        let (i, generation, reused) = self.font_slots.take(self.fonts.len(), true);
        if reused {
            self.fonts[i] = font;
            self.font_revisions[i] += 1;
        } else {
            self.fonts.push(font);
            self.font_revisions.push(0);
        }
        FontHandle::from_parts(i, generation)
    }

    /// Removes a font (its glyph table; its atlas is a texture of its own,
    /// removed with [`RenderScene::remove_texture`] after): a later add
    /// reuses the slot under a new generation, and text still set in this
    /// font is refused at the next frame ([`crate::RenderError::InvalidHandle`]).
    /// False for a handle that is not live.
    pub fn remove_font(&mut self, handle: FontHandle) -> bool {
        let i = handle.index();
        if !self.font_slots.live(i, handle.generation()) {
            return false;
        }
        let f = &mut self.fonts[i];
        f.glyphs = Vec::new();
        f.kerning = Vec::new();
        self.font_revisions[i] += 1;
        self.font_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a font of this scene that was not removed.
    pub fn contains_font(&self, handle: FontHandle) -> bool {
        self.font_slots.live(handle.index(), handle.generation())
    }

    /// Replaces a font's glyph table, kerning or atlas (glyphs rendered at
    /// run time joining a page): the renderer lays text out with the new
    /// table from the next frame. False for a bad handle.
    pub fn set_font(&mut self, handle: FontHandle, font: MsdfFont) -> bool {
        let i = handle.index();
        if !self.font_slots.live(i, handle.generation()) {
            return false;
        }
        self.fonts[i] = font;
        self.font_revisions[i] += 1;
        true
    }

    pub(crate) fn font_revisions(&self) -> &[u64] {
        &self.font_revisions
    }

    /// Adds one sprite (the same as pushing onto [`RenderScene::sprites`]).
    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    pub fn font(&self, handle: FontHandle) -> Option<&MsdfFont> {
        if !self.contains_font(handle) {
            return None;
        }
        self.fonts.get(handle.index())
    }

    /// Every per-frame reference to a removed mesh, material, texture,
    /// font, environment map, texture array, palette or LUT: instances,
    /// skinned instances, sprites (and the arrays they draw from), texts,
    /// the overlay's, the environment map, the colour grading's LUT and the
    /// stylisation's palette (nothing to look at until something was
    /// removed).
    pub(crate) fn stale_handle(&self) -> Option<String> {
        let (m, t, mat, f) = (
            &self.mesh_slots,
            &self.texture_slots,
            &self.material_slots,
            &self.font_slots,
        );
        if !(m.any_removed
            || t.any_removed
            || mat.any_removed
            || f.any_removed
            || self.environment_map_slots.any_removed
            || self.texture_array_slots.any_removed
            || self.palette_slots.any_removed
            || self.lut_slots.any_removed)
        {
            return None;
        }
        if let Some(e) = self.environment_map
            && !self.contains_environment_map(e.map)
        {
            return Some(format!("the environment map {} was removed", e.map.index()));
        }
        if let Some(g) = self.post.color_grading
            && !self.contains_lut(g.lut)
        {
            return Some(format!(
                "the colour grading's lut {} was removed",
                g.lut.index()
            ));
        }
        if let Some(p) = self.stylise.quantize.and_then(|q| q.palette)
            && !self.contains_palette(p)
        {
            return Some(format!(
                "the stylisation's palette {} was removed",
                p.index()
            ));
        }
        for s in &self.sprite_arrays {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        for s in self.overlay.iter().flat_map(|o| &o.sprite_arrays) {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "overlay sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        let draws = self
            .instances
            .iter()
            .map(|i| (i.mesh, i.material))
            .chain(self.skinned_instances.iter().map(|i| (i.mesh, i.material)));
        for (k, (mesh, material)) in draws.enumerate() {
            if !m.live(mesh.index(), mesh.generation()) {
                return Some(format!("instance {k}: mesh {} was removed", mesh.index()));
            }
            if !mat.live(material.index(), material.generation()) {
                return Some(format!(
                    "instance {k}: material {} was removed",
                    material.index()
                ));
            }
        }
        let overlay = self.overlay.as_ref();
        let sprites = self
            .sprites
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.sprites.iter()));
        for (k, s) in sprites.enumerate() {
            if let Some(h) = s.texture
                && !t.live(h.index(), h.generation())
            {
                return Some(format!("sprite {k}: texture {} was removed", h.index()));
            }
        }
        let texts = self
            .texts
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.texts.iter()));
        for (k, text) in texts.enumerate() {
            if !f.live(text.font.index(), text.font.generation()) {
                return Some(format!("text {k}: font {} was removed", text.font.index()));
            }
        }
        for (k, font) in self.fonts.iter().enumerate() {
            if f.live(k, f.generation(k)) && !t.live(font.atlas.index(), font.atlas.generation()) {
                return Some(format!(
                    "font {k}: its atlas texture {} was removed",
                    font.atlas.index()
                ));
            }
        }
        None
    }

    /// The handle of the scene's `index`-th mesh (meshes are numbered in
    /// the order they were added).
    pub fn mesh_handle(&self, index: usize) -> Option<MeshHandle> {
        let generation = self.mesh_slots.generation(index);
        self.mesh_slots
            .live(index, generation)
            .then(|| MeshHandle::from_parts(index, generation))
    }

    /// How many times the mesh at `index` was replaced (or removed).
    /// Bumped by every [`Self::set_mesh`] of the mesh at `index`.
    pub fn mesh_revision(&self, index: usize) -> u64 {
        self.mesh_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the texture at `index` was replaced (or removed).
    pub fn texture_revision(&self, index: usize) -> u64 {
        self.texture_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the terrain at `index` was replaced or edited.
    pub fn terrain_revision(&self, index: usize) -> u64 {
        self.terrain_revisions.get(index).copied().unwrap_or(0)
    }

    pub fn mesh(&self, handle: MeshHandle) -> Option<&MeshData> {
        if !self.contains_mesh(handle) {
            return None;
        }
        self.meshes.get(handle.index())
    }

    /// An RGBA8 texture (`None` for a [`TextureUpload`] or a foreign handle).
    pub fn texture(&self, handle: TextureHandle) -> Option<&TextureData> {
        match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => Some(t),
            TextureSource::Encoded(_) | TextureSource::Target(_) => None,
        }
    }

    /// A pre-encoded texture (`None` for an RGBA8 one or a foreign handle).
    pub fn texture_upload(&self, handle: TextureHandle) -> Option<&TextureUpload> {
        match self.textures.get(handle.index())? {
            TextureSource::Encoded(t) => Some(t),
            TextureSource::Rgba(_) | TextureSource::Target(_) => None,
        }
    }

    /// Width and height of any texture.
    pub fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        self.textures.get(handle.index()).map(TextureSource::size)
    }

    /// The material as added with [`RenderScene::add_material`] or
    /// [`RenderScene::set_material`]; `None` for one that is a
    /// [`StandardMaterial`] (read it with [`RenderScene::standard_material`]).
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.gltf_materials.get(handle.index())?.as_ref()
    }

    /// Any material, in the form it is drawn.
    pub fn standard_material(&self, handle: MaterialHandle) -> Option<&StandardMaterial> {
        self.materials.get(handle.index())
    }

    /// Adds a material shader (`hooks.rs`). The cook validates shader
    /// assets; one added here directly is compiled when first drawn.
    pub fn add_shader(&mut self, shader: crate::MaterialShader) -> ShaderHandle {
        self.shaders.push(shader);
        ShaderHandle(self.shaders.len() as u32 - 1)
    }

    pub fn shaders(&self) -> &[crate::MaterialShader] {
        &self.shaders
    }

    /// Draws `material` with `shader` (`None`: the built-in hooks again),
    /// its parameters at the shader's defaults. False for a bad handle.
    pub fn set_material_shader(
        &mut self,
        material: MaterialHandle,
        shader: Option<ShaderHandle>,
    ) -> bool {
        let i = material.index();
        if !self.material_slots.live(i, material.generation())
            || shader.is_some_and(|s| s.index() >= self.shaders.len())
        {
            return false;
        }
        if self.material_shaders.len() <= i {
            self.material_shaders.resize(i + 1, None);
        }
        self.material_shaders[i] = shader.map(|s| (s, self.shaders[s.index()].defaults()));
        self.material_revisions[i] += 1;
        true
    }

    /// Sets parameter `name` of `material`'s shader (as many numbers as its
    /// type takes; missing ones are 0). False when the material has no
    /// shader or the shader no such parameter.
    pub fn set_shader_param(
        &mut self,
        material: MaterialHandle,
        name: &str,
        value: &[f32],
    ) -> bool {
        let i = material.index();
        let Some(Some((shader, params))) = self.material_shaders.get_mut(i) else {
            return false;
        };
        let Some((slot, kind)) = self.shaders[shader.index()].param(name) else {
            return false;
        };
        let mut v = [0.0; 4];
        for (d, s) in v.iter_mut().zip(value.iter().take(kind.count())) {
            *d = *s;
        }
        if params[slot] != v {
            params[slot] = v;
            self.material_revisions[i] += 1;
        }
        true
    }

    /// `material`'s shader and parameters.
    pub fn material_shader(
        &self,
        material: MaterialHandle,
    ) -> Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])> {
        self.material_shaders
            .get(material.index())
            .copied()
            .flatten()
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Every material's handle, in insertion order (tools that edit all of them).
    /// Every live material's handle, in slot order.
    pub fn material_handles(&self) -> impl Iterator<Item = MaterialHandle> + use<> {
        let live: Vec<MaterialHandle> = (0..self.materials.len())
            .filter_map(|i| {
                let generation = self.material_slots.generation(i);
                self.material_slots
                    .live(i, generation)
                    .then(|| MaterialHandle::from_parts(i, generation))
            })
            .collect();
        live.into_iter()
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn meshes(&self) -> &[MeshData] {
        &self.meshes
    }

    pub(crate) fn fonts(&self) -> &[MsdfFont] {
        &self.fonts
    }

    pub(crate) fn textures(&self) -> &[TextureSource] {
        &self.textures
    }

    pub(crate) fn materials(&self) -> impl Iterator<Item = (&StandardMaterial, u64)> {
        self.materials
            .iter()
            .zip(self.material_revisions.iter().copied())
    }
}

/// How a texture is sampled where sprites draw it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    /// Bilinear, with mips when the texture has them.
    #[default]
    Linear,
    /// The nearest texel: crisp pixel art at any scale.
    Nearest,
}

/// A sprite's drop shadow ([`RenderScene::sprite_shadows`]): the sprite's
/// quad as it stands, projected along the sun onto its [`ShadowGround`]
/// (by default the plane y = 0; lifted by [`SPRITE_SHADOW_LIFT`] over a
/// ground mesh there), its texture's alpha in black at `opacity` times the
/// sprite's own alpha, faded where the ground already lies in the sun's
/// cascaded shadow (a shadow inside a shadow is no darker). With the sun
/// at or below the horizon there is none; a low sun's shadow is at most
/// five times as long as the sprite is tall. For pixel sprites standing in
/// 3D sets (a perspective camera over the xz ground); a 2D camera looking
/// down -z sees the plane edge on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpriteShadow {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    /// How dark, 0 to 1.
    pub opacity: f32,
    /// The ground the shadow falls on.
    pub ground: ShadowGround,
}

/// The ground a [`SpriteShadow`] falls on: the plane through the point
/// at `height` under the sprite's anchor with the normal `normal` (og-runtime
/// fits it to the terrain under the sprite with [`ShadowGround::fit`], or
/// takes `SpriteDesc.dropShadowHeight`), or, with `bend`, the ground's
/// heights under a grid of the shadow's points. The default is the plane
/// y = 0. A normal steeper than about 70° from up is taken as that steep.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowGround {
    pub height: f32,
    pub normal: Vec3,
    /// The shadow bent over the ground instead of laid on the plane
    /// ([`ShadowGround::fit`] sets it where the plane misses the ground
    /// under the shadow by more than [`SHADOW_BEND_TOLERANCE`]: over a
    /// hollow or a ridge narrower than the shadow).
    pub bend: Option<ShadowBend>,
}

impl Default for ShadowGround {
    fn default() -> Self {
        ShadowGround {
            height: 0.0,
            normal: Vec3::Y,
            bend: None,
        }
    }
}

/// Rows of a [`ShadowBend`]: points up the sprite's quad, so along the
/// shadow (four cells).
pub const SHADOW_BEND_ROWS: usize = 5;
/// Columns of a [`ShadowBend`]: points across the quad (two cells).
pub const SHADOW_BEND_COLUMNS: usize = 3;
/// How far (metres, plus the same share of the sprite's height) the
/// fitted plane may miss the ground under a grid point before
/// [`ShadowGround::fit`] bends the shadow.
pub const SHADOW_BEND_TOLERANCE: f32 = 0.03;

/// A drop shadow bent over uneven ground: the sprite's quad is cut into
/// 4 × 2 cells (along × across the shadow), and each corner of a cell is
/// carried down the sun to the ground's height under it, so the shadow
/// dips into a hollow and climbs over a ridge narrower than itself. The
/// renderer draws one shadow instance per cell (sprites.rs); the cells
/// share their corners exactly, so the shadow has no seams. Between the
/// points the ground is taken as straight: a feature narrower than a cell
/// (a quarter of the shadow's length, half the sprite's width) is bridged,
/// and a bump between two points can cover the shadow there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowBend {
    /// `heights[row][column]`: the ground's height (metres) where the sun
    /// carries the quad's point at (column / 2, row / 4), fractions of its
    /// width and height from the bottom-left corner as the sprite's anchor
    /// counts them, down onto the ground.
    pub heights: [[f32; SHADOW_BEND_COLUMNS]; SHADOW_BEND_ROWS],
}

impl ShadowGround {
    /// A level plane at `height`.
    pub fn level(height: f32) -> ShadowGround {
        ShadowGround {
            height,
            normal: Vec3::Y,
            bend: None,
        }
    }

    /// The plane a drop shadow of `sprite` lies on over uneven ground
    /// (`height_at(x, z)`, a terrain's height in metres), under the sun
    /// towards `direction_to_light`: through the ground at the anchor,
    /// along the slope from there to where the shadow's tip falls and
    /// across the slope over the sprite's width, raised where the ground
    /// between bulges above that plane, so no part of the shadow sinks
    /// into a hill. Where that plane misses the ground under a grid of the
    /// shadow's points by more than [`SHADOW_BEND_TOLERANCE`] (a hollow or
    /// a ridge narrower than the shadow), the shadow is bent instead
    /// ([`ShadowBend`]): each point is carried down the sun to the first
    /// ground it meets. A level plane at the anchor's ground with the sun
    /// at or below the horizon.
    pub fn fit(
        height_at: impl Fn(f32, f32) -> f32,
        sprite: &crate::Sprite,
        direction_to_light: Vec3,
    ) -> ShadowGround {
        ShadowGround::fit_facing(height_at, sprite, SpriteFacing::None, direction_to_light)
    }

    /// [`ShadowGround::fit`] for a sprite that faces the camera
    /// ([`RenderScene::sprite_facing`]): its shadow is cast by the quad
    /// stood upright facing the sun, so a bend's grid points are taken on
    /// that quad. [`SpriteFacing::None`] is `fit`.
    pub fn fit_facing(
        height_at: impl Fn(f32, f32) -> f32,
        sprite: &crate::Sprite,
        facing: SpriteFacing,
        direction_to_light: Vec3,
    ) -> ShadowGround {
        let at = glam::Vec2::new(sprite.position.x, sprite.position.z);
        let h0 = height_at(at.x, at.y);
        let Some(g) = crate::sprites::shadow_ground(direction_to_light) else {
            return ShadowGround::level(h0);
        };
        let step = glam::Vec2::new(g[1], g[2]) * sprite.size.y.abs();
        let length = step.length();
        if !(length > 1e-3 && length.is_finite()) {
            return ShadowGround::level(h0);
        }
        let along = step / length;
        let across = along.perp() * (sprite.size.x.abs() * 0.5).max(0.05);
        let mid = at + step * 0.5;
        let tip = height_at(at.x + step.x, at.y + step.y);
        let (left, right) = (
            height_at(mid.x - across.x, mid.y - across.y),
            height_at(mid.x + across.x, mid.y + across.y),
        );
        // The rise per metre along the shadow and across it, as a gradient
        // over x and z.
        let gradient = along * (tip - h0) / length
            + across.normalize() * (right - left) / (2.0 * across.length());
        let bulge = (height_at(mid.x, mid.y) - (h0 + tip) * 0.5).max(0.0);
        let mut plane = ShadowGround {
            height: h0 + bulge,
            normal: Vec3::new(-gradient.x, 1.0, -gradient.y).normalize(),
            bend: None,
        };
        let right = match facing {
            SpriteFacing::None => Vec3::X,
            SpriteFacing::Camera | SpriteFacing::CameraY => {
                crate::sprites::sun_card_right(direction_to_light)
            }
        };
        plane.bend = plane.bend_over(&height_at, sprite, right, g);
        plane
    }

    /// The bend of a shadow whose plane is `self` over `height_at`, under
    /// the sun's projection `g` (`sprites::shadow_ground`), or `None` where
    /// the plane lies within the tolerance of the ground at every point.
    /// `right`: the standing quad's x axis (+x, or the sun-facing card's of
    /// a faced sprite), its y is +y.
    fn bend_over(
        &self,
        height_at: &impl Fn(f32, f32) -> f32,
        sprite: &crate::Sprite,
        right: Vec3,
        g: [f32; 4],
    ) -> Option<ShadowBend> {
        let tall = sprite.size.y.abs().max(0.05);
        let tolerance = SHADOW_BEND_TOLERANCE * (1.0 + tall);
        let [rise_x, rise_z] = self.slope();
        let anchor = sprite.position;
        let (sin, cos) = sprite.rotation.sin_cos();
        // Each grid point as the sprite shader stands it, and where it
        // lands on the plane (sprite.wgsl sloped_shadow).
        let mut points = [[(Vec3::ZERO, 0.0); SHADOW_BEND_COLUMNS]; SHADOW_BEND_ROWS];
        let mut misses = false;
        for (row, line) in points.iter_mut().enumerate() {
            for (column, point) in line.iter_mut().enumerate() {
                let c = glam::Vec2::new(
                    column as f32 / (SHADOW_BEND_COLUMNS - 1) as f32,
                    row as f32 / (SHADOW_BEND_ROWS - 1) as f32,
                );
                let local = (c - sprite.anchor) * sprite.size;
                let (x, y) = (local.x * cos - local.y * sin, local.x * sin + local.y * cos);
                let p = if right == Vec3::X {
                    Vec3::new(anchor.x + x, anchor.y + y, anchor.z)
                } else {
                    anchor + right * x + Vec3::Y * y
                };
                // Where it lands on the plane (sprite.wgsl sloped_shadow) ...
                let above =
                    p.y - self.height - rise_x * (p.x - anchor.x) - rise_z * (p.z - anchor.z);
                let t = above / (1.0 + rise_x * g[1] + rise_z * g[2]).max(0.2);
                let on_plane = p.y - t;
                *point = (p, on_plane);
                // The ground under that landing: the plane misses it there.
                let ground = height_at(p.x + g[1] * t, p.z + g[2] * t);
                misses |= (ground - on_plane).abs() > tolerance;
            }
        }
        if !misses {
            return None;
        }
        // Bent: each point down to the first ground the sun's ray from it
        // meets.
        Some(ShadowBend {
            heights: points.map(|line| {
                line.map(|(p, on_plane)| landing(height_at, p, g, tall).unwrap_or(on_plane))
            }),
        })
    }
}

/// The height of the first ground a ray from `p` down the sun's
/// projection `g` meets (`p + (g1, -1, g2) t`, t ≥ 0), marched in 48 steps
/// over three times the sprite's height `tall` (plus a metre) and refined
/// by bisection; `None` if it meets none there.
fn landing(height_at: &impl Fn(f32, f32) -> f32, p: Vec3, g: [f32; 4], tall: f32) -> Option<f32> {
    let above = |t: f32| (p.y - t) - height_at(p.x + g[1] * t, p.z + g[2] * t);
    if above(0.0) <= 0.0 {
        return Some(height_at(p.x, p.z));
    }
    let steps = 48;
    let dt = (3.0 * tall + 1.0) / steps as f32;
    let mut lo = 0.0;
    for k in 1..=steps {
        let hi = k as f32 * dt;
        if above(hi) <= 0.0 {
            let (mut a, mut b) = (lo, hi);
            for _ in 0..12 {
                let m = 0.5 * (a + b);
                if above(m) > 0.0 { a = m } else { b = m }
            }
            let t = 0.5 * (a + b);
            return Some(p.y - t);
        }
        lo = hi;
    }
    None
}

impl ShadowGround {
    /// How the ground rises per metre along x and z (0, 0 for a level
    /// plane, or a normal that is not finite or points down).
    pub fn slope(&self) -> [f32; 2] {
        let n = self.normal;
        if !n.is_finite() || n.y <= 1e-4 {
            return [0.0, 0.0];
        }
        // At most about 70° (a rise of about 3 per metre).
        let ny = n.y.max(0.34 * n.length());
        [-n.x / ny, -n.z / ny]
    }
}

/// How far above y = 0 a sprite's drop shadow lies (metres), so it never
/// fights a ground mesh at 0 for depth.
pub const SPRITE_SHADOW_LIFT: f32 = 0.01;

/// A sprite's sort key within its layer ([`RenderScene::sprite_order`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteOrder {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    /// Lower keys draw first (under); top-down games use `-y` of the feet.
    pub key: f32,
}

/// A world text turned to the camera ([`RenderScene::text_facing`]): it
/// turns as one panel about its pivot, `position` moved forward along each
/// line's baseline by `align` of that line's width (0 the line's start,
/// 0.5 its middle, 1 its end; each line is moved back by as much so the
/// pivot stays put, as an aligned text's lines are), and draws among the
/// transparent meshes by the pivot's view depth, as a billboard does
/// ([`RenderScene::sprite_facing`]). It is not culled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFacing {
    /// The index in [`RenderScene::texts`].
    pub text: u32,
    pub facing: SpriteFacing,
    pub align: f32,
}

/// An outline around one world text ([`RenderScene::text_outlines`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextOutline {
    /// The index in [`RenderScene::texts`].
    pub text: u32,
    /// The width as a fraction of the font's distance range (1: the widest
    /// the atlas holds, half its `distance_range` in atlas pixels).
    pub width: f32,
    /// Linear RGBA.
    pub color: [f32; 4],
}

/// One quad of a tile chunk, in the layer's local coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileQuad {
    /// The bottom-left corner.
    pub min: glam::Vec2,
    pub size: glam::Vec2,
    /// `[u0, v0, u1, v1]`, v down (a flipped tile swaps u or v).
    pub uv: [f32; 4],
}

/// An animated quad of a chunk: its uv cycles through `uvs`, each held for
/// its duration (seconds), looping, by [`TileLayer::time`].
#[derive(Clone, Debug, PartialEq)]
pub struct TileAnimation {
    /// The index in [`TileChunk::quads`].
    pub quad: u32,
    pub uvs: std::sync::Arc<[[f32; 4]]>,
    pub durations: std::sync::Arc<[f32]>,
}

/// A square of tiles, culled as one against the view.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileChunk {
    /// The chunk's bounds in the layer's local coordinates.
    pub min: glam::Vec2,
    pub max: glam::Vec2,
    pub quads: Vec<TileQuad>,
    pub animated: Vec<TileAnimation>,
}

/// A layer of tiles drawn through the sprite batch: the chunks inside the
/// (orthographic) view, placed at `origin`, before the sprites of `layer`.
/// The chunks are shared (`Arc`), so a frame that resubmits an unchanged
/// layer copies no tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct TileLayer {
    pub texture: TextureHandle,
    pub chunks: std::sync::Arc<Vec<TileChunk>>,
    /// Where the layer's local origin is drawn (parallax already applied).
    pub origin: Vec3,
    pub layer: i32,
    /// Linear RGBA multiplied with every tile.
    pub color: [f32; 4],
    /// Repeat the layer every this many world units along x and y (a
    /// scrolling background); `None` draws it once.
    pub repeat: [Option<f32>; 2],
    /// Seconds of animated-tile time.
    pub time: f32,
    /// Taken by 2D lighting ([`crate::Lighting2d`]) when `layer` is in its
    /// `min_layer..=max_layer`, as the sprites of that layer are: the
    /// ambient, every [`crate::Light2d`] and the occluders' shadows. False
    /// keeps the layer as authored in any range (a sky, a far parallax
    /// backdrop).
    pub lit: bool,
}

/// The nine pieces of a nine-slice sprite: the corners keep their size, the
/// edges stretch along one axis and the centre along both. `insets` are
/// `[left, top, right, bottom]` in texture pixels of the sprite's region
/// (its `uv_rect` in a texture of `texture_size`), drawn at `world_per_pixel`
/// world units each (clamped to half the sprite). A flipped region (u0 > u1)
/// stays flipped. Empty pieces are left out.
pub fn nine_slice(
    sprite: &Sprite,
    insets: [f32; 4],
    texture_size: (u32, u32),
    world_per_pixel: f32,
) -> Vec<Sprite> {
    let [u0, v0, u1, v1] = sprite.uv_rect;
    let region = glam::Vec2::new(
        (u1 - u0).abs() * texture_size.0 as f32,
        (v1 - v0).abs() * texture_size.1 as f32,
    );
    let size = sprite.size;
    let half = size * 0.5;
    let [l, t, r, b] = insets.map(|v| v.max(0.0) * world_per_pixel);
    let (l, r) = (l.min(half.x), r.min(half.x));
    let (t, b) = (t.min(half.y), b.min(half.y));
    let frac = |px: f32, span: f32| {
        if span > 0.0 {
            (px / span).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    let wpp = world_per_pixel.max(1e-9);
    // Splits along x (world, from the left edge) and u; along y (world, from
    // the bottom edge) and v (v runs down: the bottom of the quad is v1).
    let xs = [0.0, l, size.x - r, size.x];
    let us = [
        u0,
        u0 + (u1 - u0) * frac(l / wpp, region.x),
        u1 - (u1 - u0) * frac(r / wpp, region.x),
        u1,
    ];
    let ys = [0.0, b, size.y - t, size.y];
    let vs = [
        v1,
        v1 + (v0 - v1) * frac(b / wpp, region.y),
        v0 - (v0 - v1) * frac(t / wpp, region.y),
        v0,
    ];
    let (sin, cos) = sprite.rotation.sin_cos();
    let origin = -sprite.anchor * size;
    let mut out = Vec::with_capacity(9);
    for j in 0..3 {
        for i in 0..3 {
            let (w, h) = (xs[i + 1] - xs[i], ys[j + 1] - ys[j]);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            let local = origin + glam::Vec2::new(xs[i], ys[j]);
            let rotated =
                glam::Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
            out.push(Sprite {
                position: sprite.position + rotated.extend(0.0),
                size: glam::Vec2::new(w, h),
                anchor: glam::Vec2::ZERO,
                uv_rect: [us[i], vs[j + 1], us[i + 1], vs[j]],
                ..*sprite
            });
        }
    }
    out
}
