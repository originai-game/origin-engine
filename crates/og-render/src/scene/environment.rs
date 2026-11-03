//! Scene instances, camera, lights and post effects.

use super::*;

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
/// read by the lit shaders on every backend (pbr_fs.wgsl, physical.wgsl):
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
pub use crate::shadows::MAX_SHADOWED_LIGHTS;

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
    /// shadowed lights in view, the [`crate::RenderInfo::shadowed_lights`]
    /// (8) with the most radiance at the camera get them; the rest are
    /// counted in
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

/// The sky cube drawn on a dome with a flat ground under it instead of at
/// infinity, as three.js `GroundedSkybox`: an environment photographed
/// `height` above the ground shows that ground where it lies, so objects
/// stand on it rather than float over a horizon. The dome's radius is
/// `radius`; its ground disc is where the dome would pass under the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundProjection {
    /// How far above the ground the map was taken (metres, above 0).
    pub height: f32,
    /// The dome's radius (metres, larger than `height`).
    pub radius: f32,
    /// The ground point under where the map was taken (the dome's centre
    /// is `height` above it).
    pub center: Vec3,
}

impl Default for GroundProjection {
    fn default() -> Self {
        GroundProjection {
            height: 15.0,
            radius: 100.0,
            center: Vec3::ZERO,
        }
    }
}

/// The sky pass's uniform (sky.wgsl `SkyParams`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkyParams {
    /// xyz: the grounded dome's centre (the frame's coordinates), w: its
    /// height above the ground (0: no dome).
    pub dome: [f32; 4],
    /// x: the dome's radius, y: the blurriness's prefiltered level (0:
    /// the sharp sky cube), z: what the prefiltered cube is scaled by to
    /// match the sky cube (the background intensity of a map).
    pub look: [f32; 4],
}

impl SkyParams {
    /// `env`'s background as the sky pass draws it: `levels` the
    /// prefiltered cube's mip count, `scale` the sky cube's scale over it.
    pub(crate) fn new(env: &Environment, levels: u32, scale: f32) -> SkyParams {
        let blur = if env.background_blurriness.is_finite() {
            env.background_blurriness.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let ground = env
            .ground
            .filter(|g| g.height > 0.0 && g.radius > g.height && g.radius.is_finite());
        let (dome, radius) = match ground {
            Some(g) => ((g.center + Vec3::Y * g.height).extend(g.height), g.radius),
            None => (glam::Vec4::ZERO, 0.0),
        };
        SkyParams {
            dome: dome.to_array(),
            look: [radius, blur * levels.saturating_sub(1) as f32, scale, 0.0],
        }
    }
}

/// Environment lighting and background.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    pub sky: ProceduralSky,
    /// Strength of the image-based (ambient) light.
    pub intensity: f32,
    pub background: Background,
    /// How blurred the sky cube is where it is drawn behind the scene, as
    /// three.js `scene.backgroundBlurriness`: 0 sharp to 1 the roughest of
    /// the prefiltered (GGX) levels the light already reads; the light is
    /// unchanged. For [`Background::Sky`].
    pub background_blurriness: f32,
    /// The sky cube drawn on a grounded dome ([`GroundProjection`]); `None`
    /// at infinity. For [`Background::Sky`].
    pub ground: Option<GroundProjection>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            sky: ProceduralSky::default(),
            intensity: 0.55,
            background: Background::Sky,
            background_blurriness: 0.0,
            ground: None,
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
    /// A zoom, spin or directional blur of the lit frame (an HDR pass
    /// after motion blur).
    pub screen_blur: Option<crate::ScreenBlur>,
    pub film_grain: Option<FilmGrain>,
    /// Contrast-adaptive sharpening of the display image (three.js-style CAS
    /// over the graded, sRGB-encoded colour), its strength 0 to 1.
    pub sharpen: Option<f32>,
    /// A full-screen hit flash.
    pub flash: Option<ScreenFlash>,
    /// Screen-space reflections (`None`: off).
    pub ssr: Option<crate::ScreenSpaceReflections>,
    /// Screen-space global illumination (`None`: off; `ssgi.rs`).
    pub gi: Option<crate::ScreenSpaceGlobalIllumination>,
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

#[cfg(test)]
mod sky_params_tests {
    use super::*;

    /// The sky pass's uniform: blurriness 0..1 maps onto the prefiltered
    /// levels (1 the roughest, out-of-range clamped), and a dome draws
    /// only with a positive height and a radius past it, its centre
    /// `height` above the ground point.
    #[test]
    fn sky_params_map_blurriness_and_the_dome() {
        let env = |blur: f32, ground: Option<GroundProjection>| Environment {
            background_blurriness: blur,
            ground,
            ..Environment::default()
        };
        let p = SkyParams::new(&env(0.0, None), 5, 1.0);
        assert_eq!(
            p,
            SkyParams {
                dome: [0.0; 4],
                look: [0.0, 0.0, 1.0, 0.0]
            }
        );
        assert_eq!(
            SkyParams::new(&env(1.0, None), 5, 0.5).look[1..3],
            [4.0, 0.5]
        );
        assert_eq!(SkyParams::new(&env(0.5, None), 5, 1.0).look[1], 2.0);
        assert_eq!(SkyParams::new(&env(3.0, None), 5, 1.0).look[1], 4.0);
        assert_eq!(SkyParams::new(&env(f32::NAN, None), 5, 1.0).look[1], 0.0);
        let g = GroundProjection {
            height: 2.0,
            radius: 30.0,
            center: Vec3::new(1.0, -1.0, 3.0),
        };
        let p = SkyParams::new(&env(0.0, Some(g)), 5, 1.0);
        assert_eq!(p.dome, [1.0, 1.0, 3.0, 2.0]);
        assert_eq!(p.look[0], 30.0);
        for bad in [
            GroundProjection { height: 0.0, ..g },
            GroundProjection { radius: 2.0, ..g },
            GroundProjection {
                radius: f32::INFINITY,
                ..g
            },
        ] {
            assert_eq!(SkyParams::new(&env(0.0, Some(bad)), 5, 1.0).dome, [0.0; 4]);
        }
    }
}
