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
    DepthOfField, FilmGrain, Glow, GodRays, Light2d, Lighting2d, LutHandle, MotionBlur, Occluder2d,
    Outline, ScreenFlash, SpriteFx, Vignette,
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
}

impl TextureSource {
    /// Width and height of level 0.
    pub fn size(&self) -> (u32, u32) {
        match self {
            TextureSource::Rgba(t) => (t.width, t.height),
            TextureSource::Encoded(t) => (t.desc.width, t.desc.height),
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

macro_rules! handle {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u32);
        impl $name {
            /// The position of the resource in its scene, in insertion order.
            pub fn index(self) -> usize {
                self.0 as usize
            }
            #[allow(dead_code)]
            pub(crate) fn from_index(index: usize) -> Self {
                $name(index as u32)
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

/// One object to draw: a mesh with a material at a world transform. Callers
/// submit instances one by one; the renderer batches all instances that share
/// a (mesh, material) pair into one instanced draw per pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    /// Local to world. Normals use the upper 3x3 directly, so keep scale uniform.
    pub transform: Mat4,
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
    pub vignette: Option<Vignette>,
    pub chromatic_aberration: Option<ChromaticAberration>,
    pub film_grain: Option<FilmGrain>,
    /// A full-screen hit flash.
    pub flash: Option<ScreenFlash>,
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

impl Overlay {
    /// An empty overlay of `width`×`height` CSS pixels at `scale` device
    /// pixels per CSS pixel.
    pub fn new(width: f32, height: f32, scale: f32) -> Overlay {
        Overlay {
            sprites: Vec::new(),
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
    environment_maps: Vec<crate::EnvironmentMapData>,
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
    pub post: PostEffects,
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
    pub trails: Vec<Trail>,
    pub ribbons: Vec<Ribbon>,
    pub beams: Vec<Beam>,
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
    /// Screen shake applied to the camera.
    pub shake: CameraShake,
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
    /// How each texture is sampled by sprites, tiles and text (by texture index).
    texture_filters: Vec<TextureFilter>,
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
            light: DirectionalLight::default(),
            lights: Vec::new(),
            environment: Environment::default(),
            environment_map: None,
            post: PostEffects::default(),
            sprites: Vec::new(),
            texts: Vec::new(),
            debug: DebugView::None,
            skins: Vec::new(),
            morphs: Vec::new(),
            emitters: Vec::new(),
            luts: Vec::new(),
            environment_maps: Vec::new(),
            trail_styles: Vec::new(),
            terrain_data: Vec::new(),
            terrain_revisions: Vec::new(),
            tick: 0,
            tick_rate: 60.0,
            particles: Vec::new(),
            trails: Vec::new(),
            ribbons: Vec::new(),
            beams: Vec::new(),
            decals: Vec::new(),
            decal_atlas: None,
            lighting_2d: None,
            lights_2d: Vec::new(),
            occluders_2d: Vec::new(),
            sprite_fx: Vec::new(),
            shake: CameraShake::default(),
            previous_transforms: Vec::new(),
            joint_palettes: Vec::new(),
            skinned_instances: Vec::new(),
            overlay: None,
            fog: None,
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
            texture_filters: Vec::new(),
        }
    }

    /// Samples a texture of this scene nearest (pixel art) or linear (the
    /// default) wherever sprites, tiles and particles draw it. `false` for
    /// a handle of another scene.
    pub fn set_texture_filter(&mut self, texture: TextureHandle, filter: TextureFilter) -> bool {
        let i = texture.index();
        if i >= self.textures.len() {
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
        let h = self.add_mesh(mesh);
        self.skins[h.index()] = Some(skin);
        h
    }

    /// Gives a mesh blendshape targets (one delta per vertex per target, at
    /// most [`crate::MAX_MORPH_TARGETS`]); draw it with
    /// [`RenderScene::skinned_instances`] (weights per instance; a mesh
    /// without a skin stays in its bind pose). Checked at upload. Set them
    /// before the mesh's first frame; `false` for a handle of another scene.
    pub fn set_morph_targets(&mut self, mesh: MeshHandle, targets: MorphTargets) -> bool {
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
    ) -> Option<&std::collections::BTreeMap<u32, std::sync::Arc<crate::TerrainTile>>> {
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

    /// Adds a 32³ colour grading table ([`ColorLut`]).
    pub fn add_lut(&mut self, lut: ColorLut) -> LutHandle {
        self.luts.push(lut);
        LutHandle(self.luts.len() as u32 - 1)
    }

    /// Adds a prefiltered environment map (an asset; og-cook's `type
    /// environment`); [`RenderScene::environment_map`] picks the one a
    /// frame is lit by.
    pub fn add_environment_map(&mut self, map: crate::EnvironmentMapData) -> EnvironmentMapHandle {
        self.environment_maps.push(map);
        EnvironmentMapHandle(self.environment_maps.len() as u32 - 1)
    }

    pub(crate) fn environment_maps(&self) -> &[crate::EnvironmentMapData] {
        &self.environment_maps
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

    pub(crate) fn trail_styles(&self) -> &[TrailStyle] {
        &self.trail_styles
    }

    pub(crate) fn skins(&self) -> &[Option<SkinData>] {
        &self.skins
    }

    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshHandle {
        self.meshes.push(mesh);
        self.skins.push(None);
        self.morphs.push(None);
        self.mesh_revisions.push(0);
        MeshHandle(self.meshes.len() as u32 - 1)
    }

    /// Replaces a mesh's vertices and indices (cloth, generated geometry);
    /// the renderer uploads it again before the next frame, in place when
    /// the sizes match. False for a bad handle or a skinned or morphed mesh.
    pub fn set_mesh(&mut self, handle: MeshHandle, mesh: MeshData) -> bool {
        let i = handle.index();
        if i >= self.meshes.len() || self.skins[i].is_some() || self.morphs[i].is_some() {
            return false;
        }
        self.meshes[i] = mesh;
        self.mesh_revisions[i] += 1;
        true
    }

    pub(crate) fn mesh_revisions(&self) -> &[u64] {
        &self.mesh_revisions
    }

    /// Adds an RGBA8 texture (mips generated on the GPU when asked).
    pub fn add_texture(&mut self, texture: TextureData) -> TextureHandle {
        self.textures.push(TextureSource::Rgba(texture));
        self.texture_revisions.push(0);
        TextureHandle(self.textures.len() as u32 - 1)
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
        if i >= self.textures.len() {
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
        self.textures.push(TextureSource::Encoded(upload));
        self.texture_revisions.push(0);
        TextureHandle(self.textures.len() as u32 - 1)
    }

    /// Adds a texture of either kind ([`TextureSource`]).
    pub fn add_texture_source(&mut self, source: TextureSource) -> TextureHandle {
        self.textures.push(source);
        self.texture_revisions.push(0);
        TextureHandle(self.textures.len() as u32 - 1)
    }

    /// Adds a glTF-shaped material (drawn as its [`StandardMaterial`] form).
    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.push(StandardMaterial::from(&material));
        self.gltf_materials.push(Some(material));
        self.material_revisions.push(0);
        MaterialHandle(self.materials.len() as u32 - 1)
    }

    /// Adds a three.js-style standard material.
    pub fn add_standard_material(&mut self, material: StandardMaterial) -> MaterialHandle {
        self.materials.push(material);
        self.gltf_materials.push(None);
        self.material_revisions.push(0);
        MaterialHandle(self.materials.len() as u32 - 1)
    }

    /// Replaces a material with a glTF-shaped one; the renderer re-uploads it
    /// on the next frame. Returns `false` (and changes nothing) for a handle
    /// of another scene.
    pub fn set_material(&mut self, handle: MaterialHandle, material: Material) -> bool {
        let i = handle.index();
        if i >= self.materials.len() {
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
        if i >= self.materials.len() {
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
        });
    }

    /// Adds an MSDF font whose atlas is a texture of this scene.
    pub fn add_font(&mut self, font: MsdfFont) -> FontHandle {
        self.fonts.push(font);
        self.font_revisions.push(0);
        FontHandle(self.fonts.len() as u32 - 1)
    }

    /// Replaces a font's glyph table, kerning or atlas (glyphs rendered at
    /// run time joining a page): the renderer lays text out with the new
    /// table from the next frame. False for a bad handle.
    pub fn set_font(&mut self, handle: FontHandle, font: MsdfFont) -> bool {
        let i = handle.index();
        if i >= self.fonts.len() {
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
        self.fonts.get(handle.index())
    }

    /// The handle of the scene's `index`-th mesh (meshes are numbered in
    /// the order they were added).
    pub fn mesh_handle(&self, index: usize) -> Option<MeshHandle> {
        (index < self.meshes.len()).then(|| MeshHandle::from_index(index))
    }

    pub fn mesh(&self, handle: MeshHandle) -> Option<&MeshData> {
        self.meshes.get(handle.index())
    }

    /// An RGBA8 texture (`None` for a [`TextureUpload`] or a foreign handle).
    pub fn texture(&self, handle: TextureHandle) -> Option<&TextureData> {
        match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => Some(t),
            TextureSource::Encoded(_) => None,
        }
    }

    /// A pre-encoded texture (`None` for an RGBA8 one or a foreign handle).
    pub fn texture_upload(&self, handle: TextureHandle) -> Option<&TextureUpload> {
        match self.textures.get(handle.index())? {
            TextureSource::Encoded(t) => Some(t),
            TextureSource::Rgba(_) => None,
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
        if i >= self.materials.len() || shader.is_some_and(|s| s.index() >= self.shaders.len()) {
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
    pub fn material_handles(&self) -> impl Iterator<Item = MaterialHandle> + use<> {
        (0..self.materials.len()).map(MaterialHandle::from_index)
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

/// A sprite's sort key within its layer ([`RenderScene::sprite_order`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteOrder {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    /// Lower keys draw first (under); top-down games use `-y` of the feet.
    pub key: f32,
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
