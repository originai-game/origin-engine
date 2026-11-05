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
    /// How far a foliage mesh ([`RenderScene::set_mesh_foliage`]) sways in
    /// [`RenderScene::wind`]: its wind strength times this (0: still, 1:
    /// the scene's wind). Carried in the instance record's layer word for
    /// runs without array maps (`instances.rs`); a mesh without foliage
    /// ignores it. [`RenderScene::add_instance`] keeps still.
    pub sway: f32,
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
    /// Which object the outline pass takes it for: 0, its own (told
    /// from its transform); otherwise every instance with the same group
    /// is one object, lined round together and never along the seams
    /// between them (a fractured prop's chunks still bonded into one
    /// island). Folded to 255 ids, as transforms are.
    pub outline: u32,
}

impl InstanceFx {
    /// No flash, a white tint: the material's own look.
    pub const NONE: InstanceFx = InstanceFx {
        flash: 0.0,
        flash_color: Vec3::ONE,
        tint: Vec3::ONE,
        outline: 0,
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
    /// How much of the sun's light a cloud cover or the weather has
    /// scattered across the sky, 0 (a clear sky, the default) to 1 (an
    /// overcast): what is left of the beam then comes from a wide patch of
    /// cloud round the sun, so its shadow softens (PCSS at up to a quarter
    /// metre of penumbra per metre behind the caster) and fades to 0.35 of
    /// its depth where the sky round the sun is open, as the rest of the
    /// scattered light reaches the shadow too (under a roof it stays
    /// shade). [`DirectionalLight::cover`] sets it.
    pub diffusion: f32,
}

impl DirectionalLight {
    /// Dims the light under a cover that passes `beam` (0..1) of it, as
    /// clouds and the weather do ([`crate::Clouds::light_scene`],
    /// [`crate::WeatherLook::light`]): its intensity times `beam`, and the
    /// rest counted as scattered ([`DirectionalLight::diffusion`]), from
    /// none while the cover takes less than 15 % to all of it past 85 %
    /// (smoothly between). Two covers keep the greater diffusion.
    pub fn cover(&mut self, beam: f32) {
        let beam = beam.clamp(0.0, 1.0);
        self.intensity *= beam;
        let t = ((1.0 - beam - 0.15) / 0.7).clamp(0.0, 1.0);
        self.diffusion = self.diffusion.max(t * t * (3.0 - 2.0 * t));
    }

    /// What the frame's shadows read of [`DirectionalLight::diffusion`]:
    /// the PCSS softness (the declared one, or the scattered light's
    /// wider, 0.25 at full diffusion) and how much of the shadow's depth
    /// it fills in (0.65 at full diffusion).
    pub(crate) fn shadow_softness_and_fill(&self) -> (f32, f32) {
        let d = self.diffusion.clamp(0.0, 1.0);
        (self.shadow_softness.max(0.0).max(0.25 * d), 0.65 * d)
    }
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
            diffusion: 0.0,
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
    /// `sun_color`. Without an environment map the image-based light and
    /// the reflections' fallback follow this sky live, clouds included:
    /// captured a slice a frame, a full refresh every eight frames
    /// (`sky_capture.rs`). Stars are drawn here only.
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

impl GroundProjection {
    /// The frame's `env_dome` (frame.wgsl `dome_lookup`, read by the sky
    /// pass and by every reflection of the environment): xyz the dome's
    /// centre (the frame's coordinates), w its height above the ground, then
    /// (radius, 0, 0, 0); all zero (no dome) without a positive height and
    /// a finite radius past it.
    pub(crate) fn gpu(this: Option<GroundProjection>) -> [[f32; 4]; 2] {
        match this.filter(|g| g.height > 0.0 && g.radius > g.height && g.radius.is_finite()) {
            Some(g) => [
                (g.center + Vec3::Y * g.height).extend(g.height).to_array(),
                [g.radius, 0.0, 0.0, 0.0],
            ],
            None => [[0.0; 4]; 2],
        }
    }
}

/// The sky pass's uniform (sky.wgsl `SkyParams`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SkyParams {
    /// x: 1 when the background is an environment map drawn from its own
    /// cube at its full size (`map_cube`), y: the blurriness's prefiltered
    /// level (0: sharp), z: the map's background intensity, w: the blended
    /// map's weight.
    pub look: [f32; 4],
    /// (cos, sin) of the map's turn about +y; zw unused.
    pub turn: [f32; 4],
}

/// The map the sky pass draws as the background: its turn, its
/// background intensity and the drawn weight of the map it blends towards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SkyMap {
    pub rotation: f32,
    pub scale: f32,
    pub blend: f32,
}

impl SkyParams {
    /// `env`'s background as the sky pass draws it: `levels` the
    /// prefiltered cube's mip count, `map` the environment map drawn (none:
    /// the sky cube).
    pub(crate) fn new(env: &Environment, levels: u32, map: Option<SkyMap>) -> SkyParams {
        let blur = if env.background_blurriness.is_finite() {
            env.background_blurriness.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let level = blur * levels.saturating_sub(1) as f32;
        match map {
            Some(m) => SkyParams {
                look: [1.0, level, m.scale, m.blend],
                turn: [m.rotation.cos(), m.rotation.sin(), 0.0, 0.0],
            },
            None => SkyParams {
                look: [0.0, level, 1.0, 0.0],
                turn: [1.0, 0.0, 0.0, 0.0],
            },
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
    /// Lens flare after bloom (`None`: off, no pass recorded).
    pub lens_flare: Option<crate::LensFlare>,
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
    /// Probe global illumination: light bounced through the whole scene and
    /// the sky occluded by it, from camera-centred probes traced in
    /// software (`None`: off; `gi_probes.rs`). Beside or instead of
    /// [`PostEffects::gi`].
    pub gi_probes: Option<crate::ProbeGlobalIllumination>,
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
    /// Terrain triplanar projection weights (XYZ as RGB), with clipmap
    /// cell edges dark and the outer morph band white.
    Terrain,
    /// Terrain's strongest slope/control layer, coloured by its index.
    TerrainLayers,
    /// Terrain normal-map mip footprint, one colour band per level.
    TerrainMip,
    /// Dominant raw terrain layer's XYZ projection weights, without overlays.
    TerrainProjection,
    /// Dominant raw layer's XYZ vertical texel densities, encoded as d/(1+d).
    TerrainDensity,
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
            DebugView::Terrain => 7.0,
            DebugView::TerrainLayers => 8.0,
            DebugView::TerrainMip => 9.0,
            DebugView::TerrainProjection => 10.0,
            DebugView::TerrainDensity => 11.0,
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
    /// levels (1 the roughest, out-of-range clamped), and a map is drawn
    /// from its own cube with its turn, scale and blend.
    #[test]
    fn sky_params_map_blurriness_and_the_map() {
        let env = |blur: f32| Environment {
            background_blurriness: blur,
            ..Environment::default()
        };
        assert_eq!(
            SkyParams::new(&env(0.0), 5, None),
            SkyParams {
                look: [0.0, 0.0, 1.0, 0.0],
                turn: [1.0, 0.0, 0.0, 0.0]
            }
        );
        assert_eq!(SkyParams::new(&env(1.0), 5, None).look[1], 4.0);
        assert_eq!(SkyParams::new(&env(0.5), 5, None).look[1], 2.0);
        assert_eq!(SkyParams::new(&env(3.0), 5, None).look[1], 4.0);
        assert_eq!(SkyParams::new(&env(f32::NAN), 5, None).look[1], 0.0);
        let m = SkyMap {
            rotation: std::f32::consts::FRAC_PI_2,
            scale: 0.5,
            blend: 0.25,
        };
        let p = SkyParams::new(&env(0.0), 5, Some(m));
        assert_eq!(p.look, [1.0, 0.0, 0.5, 0.25]);
        assert!(p.turn[0].abs() < 1e-6 && (p.turn[1] - 1.0).abs() < 1e-6);
    }

    /// A dome draws only with a positive height and a finite radius past
    /// it, its centre `height` above the ground point.
    #[test]
    fn the_dome_is_drawn_only_when_it_can_be() {
        let g = GroundProjection {
            height: 2.0,
            radius: 30.0,
            center: Vec3::new(1.0, -1.0, 3.0),
        };
        assert_eq!(
            GroundProjection::gpu(Some(g)),
            [[1.0, 1.0, 3.0, 2.0], [30.0, 0.0, 0.0, 0.0]]
        );
        assert_eq!(GroundProjection::gpu(None), [[0.0; 4]; 2]);
        for bad in [
            GroundProjection { height: 0.0, ..g },
            GroundProjection { radius: 2.0, ..g },
            GroundProjection {
                radius: f32::INFINITY,
                ..g
            },
        ] {
            assert_eq!(GroundProjection::gpu(Some(bad)), [[0.0; 4]; 2]);
        }
    }

    /// A cover's scattered light reaches the sun's shadow: none while it
    /// takes under 15 % (a broken sky's frames stay as they were), most of
    /// the weather's rain dimming (0.35 left: a smoothstep at 5/7, 0.8016),
    /// all of an overcast's; the PCSS softness and the fill follow, and a
    /// second, thinner cover keeps the first's.
    #[test]
    fn a_cover_softens_and_fills_the_suns_shadow() {
        let mut fair = DirectionalLight::default();
        fair.cover(0.9);
        assert_eq!(fair.diffusion, 0.0);
        assert!((fair.intensity - 2.7).abs() < 1e-6);
        assert_eq!(fair.shadow_softness_and_fill(), (0.0, 0.0));

        let mut rain = DirectionalLight::default();
        rain.cover(0.35);
        assert!((rain.diffusion - 0.8016).abs() < 1e-3, "{}", rain.diffusion);
        let (softness, fill) = rain.shadow_softness_and_fill();
        assert!((softness - 0.2004).abs() < 1e-3 && (fill - 0.5210).abs() < 1e-3);
        rain.cover(0.95);
        assert!((rain.diffusion - 0.8016).abs() < 1e-3);
        assert!((rain.intensity - 3.0 * 0.35 * 0.95).abs() < 1e-5);

        let mut overcast = crate::RenderScene::new();
        crate::Clouds {
            coverage: 1.0,
            ..crate::Clouds::default()
        }
        .light_scene(&mut overcast);
        assert!(overcast.light.diffusion > 0.999);
        assert!((overcast.light.shadow_softness_and_fill().1 - 0.65).abs() < 1e-3);

        // A declared softness wider than the cover's is kept.
        let mut soft = DirectionalLight {
            shadow_softness: 0.3,
            ..Default::default()
        };
        soft.cover(0.0);
        assert_eq!(soft.shadow_softness_and_fill(), (0.3, 0.65));
    }
}
