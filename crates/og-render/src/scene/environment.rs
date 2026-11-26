//! Scene instances, camera, lights and post effects.

use super::*;
use glam::Vec2;
pub use og_render_data::camera::{
    COC_SOURCE_CAPACITY, CocMix, CocSource, GateFit, PhysicalLens, WeightedCoc,
};

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
/// channel. `reveal` fades the instance in: the share of its pixels drawn,
/// picked by a screen-space dither in every pass that draws it (lit,
/// prepass and shadow), in 255 steps. [`InstanceFx::NONE`] (the default)
/// shades exactly as the material does.
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
    /// How much of it is drawn, 0 to 1 (1: whole, the default): a model
    /// fading in draws this share of its pixels, dithered in screen space
    /// (common.wgsl `dither_cut`), and its shadow the same share of its
    /// texels. At 0 nothing of it is drawn, and the renderer makes the
    /// pipelines it will draw with on its compile workers meanwhile
    /// ([`crate::FrameStats::compiling_ahead`]): a producer that keeps it
    /// at 0 until that is false never stops a frame for them.
    /// Presentation only: the simulation never reads it.
    pub reveal: f32,
    /// Whether it fades out against another fading in (a cross-fade): its
    /// dither keeps exactly the pixels and shadow texels an instance with
    /// `reveal` `1 - reveal` and no `fade_out` cuts, so the pair draws
    /// each pixel once, the one or the other (og-runtime's HLOD draws a
    /// proxy and the models it stands in for so). Read only while
    /// `reveal` is under 1.
    pub fade_out: bool,
}

impl InstanceFx {
    /// No flash, a white tint: the material's own look.
    pub const NONE: InstanceFx = InstanceFx {
        flash: 0.0,
        flash_color: Vec3::ONE,
        tint: Vec3::ONE,
        outline: 0,
        reveal: 1.0,
        fade_out: false,
    };

    /// [`Self::reveal`] as the GPU reads it: how much is hidden, in 255
    /// steps (0: whole). A model that is not whole is drawn apart from
    /// those that are (`instances.rs`: its run keeps the discard).
    pub(crate) fn hidden_steps(&self) -> u32 {
        if self.reveal.is_finite() {
            ((1.0 - self.reveal.clamp(0.0, 1.0)) * 255.0 + 0.5) as u32
        } else {
            0
        }
    }

    /// [`Self::hidden_steps`] with [`Self::fade_out`] in bit 8 (set only
    /// while some of it is hidden): what the instance record carries
    /// (instance.wgsl `look_hidden_steps`).
    pub(crate) fn hidden_bits(&self) -> u32 {
        match self.hidden_steps() {
            0 => 0,
            steps => steps | (u32::from(self.fade_out) << 8),
        }
    }
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
    /// Physical or blended perspective, with a shifted sensor window.
    /// Positive shift moves the viewed window right/up, not the pose.
    PerspectiveShifted { fov_y_degrees: f32, shift: Vec2 },
    /// Orthographic: `height` world units fill the target vertically; the
    /// width follows the target's aspect ratio. The 2D camera mode.
    Orthographic { height: f32 },
}

impl Projection {
    /// The projection on a target `aspect` wide per unit tall, depth `near`
    /// to `far` onto 0..1 (og-core's [`og_core::math::perspective`] and
    /// [`og_core::math::orthographic`]): what the frame, the shadow
    /// cascades and the debug labels project with.
    pub fn matrix(&self, aspect: f32, near: f32, far: f32) -> glam::Mat4 {
        match *self {
            Projection::Perspective { fov_y_degrees } => {
                og_core::math::perspective(fov_y_degrees.to_radians(), aspect, near, far)
            }
            Projection::PerspectiveShifted {
                fov_y_degrees,
                shift,
            } => {
                let (s, c) = og_core::math::sin_cos(fov_y_degrees.to_radians() * 0.5);
                og_core::math::perspective_shifted(c / s, aspect, near, far, shift)
            }
            Projection::Orthographic { height } => {
                let h = height.abs().max(1e-6) * 0.5;
                og_core::math::orthographic(h * aspect, h, near, far)
            }
        }
    }
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
    /// brought to the key that suits it (fullscreen.wgsl `metered_key`: 0.18,
    /// middle grey, for a daylit view, higher for a brighter view and lower
    /// for a dimmer one), then `compensation` is added and the result is
    /// clamped to `min_ev..=max_ev`.
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

pub use og_render_data::light::{
    DirectionalLight, LightDesc, LightKind, LightMobility, ShadowBounds,
};

/// The directional light a frame draws from several ([`key_light`]).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyLight {
    pub light: DirectionalLight,
    /// The candidate (an index into the slice) it is, the brighter of two
    /// handing over: its shadow settings are the drawn light's.
    pub source: usize,
    /// The candidate blended in with it while the two hand over.
    pub blended: Option<usize>,
    /// `source`'s share of the drawn light: 1 out of a handover, down to
    /// 0.5 where the two are equal (`blended` has the rest).
    pub weight: f32,
    /// The lights (indices into the slice) left out of the drawn one while
    /// they shine with at least a quarter of its radiance.
    pub left_out: Vec<usize>,
}

impl KeyLight {
    /// Candidate `i`'s share of the drawn light: 0 for one left out.
    pub fn share(&self, i: usize) -> f32 {
        if i == self.source {
            self.weight
        } else if Some(i) == self.blended {
            1.0 - self.weight
        } else {
            0.0
        }
    }
}

/// The one directional light a frame draws (its cascades, the sky's sun,
/// the clouds' and the weather's cover all follow it) from the scene's
/// several, as Unity's main light: the brightest by radiance (intensity
/// times the colour's luminance), ties to the earlier. Two within 1.5× of
/// each other hand over: from where the brighter has 60 % of their
/// radiance to where they are equal, the drawn light's direction, colour
/// and intensity blend towards the other's, dimmed by how far apart the two
/// point (opposite ones dim to nothing at the crossing, as og.DayNight's
/// sun and moon do), so a sun rising as the moon fades turns the light and
/// its shadows round without a jump. The shadow settings are the
/// brighter's. A pure function of the lights; `None` without one.
pub fn key_light(lights: &[DirectionalLight]) -> Option<KeyLight> {
    // The relative difference of the two brightest below which they blend.
    const HANDOVER: f32 = 0.2;
    let radiance = |l: &DirectionalLight| {
        (l.intensity * l.color.dot(Vec3::new(0.2126, 0.7152, 0.0722))).max(0.0)
    };
    let mut order: Vec<usize> = (0..lights.len()).collect();
    // Stable: equal lights keep their order.
    order.sort_by(|&a, &b| radiance(&lights[b]).total_cmp(&radiance(&lights[a])));
    let (&first, rest) = order.split_first()?;
    let (a, ra) = (lights[first], radiance(&lights[first]));
    let mut light = a;
    let (mut blended, mut weight) = (None, 1.0);
    if let Some(&second) = rest.first() {
        let (b, rb) = (lights[second], radiance(&lights[second]));
        let q = if ra + rb > 0.0 {
            (ra - rb) / (ra + rb)
        } else {
            1.0
        };
        let t = (q / HANDOVER).clamp(0.0, 1.0);
        // The brighter's weight: a half where they are equal.
        let w = 0.5 + 0.5 * t * t * (3.0 - 2.0 * t);
        if w < 1.0 {
            (blended, weight) = (Some(second), w);
            let towards =
                |l: &DirectionalLight| l.direction_to_light.try_normalize().unwrap_or(Vec3::Y);
            let v = towards(&a) * w + towards(&b) * (1.0 - w);
            let reach = v.length();
            light.direction_to_light = if reach > 1e-6 { v / reach } else { towards(&a) };
            let intensity = a.intensity * w + b.intensity * (1.0 - w);
            if intensity > 0.0 {
                light.color =
                    (a.color * a.intensity * w + b.color * b.intensity * (1.0 - w)) / intensity;
            }
            light.intensity = intensity * reach;
        }
    }
    let left_out = rest[usize::from(blended.is_some())..]
        .iter()
        .copied()
        .filter(|&i| radiance(&lights[i]) > 0.0 && radiance(&lights[i]) >= 0.25 * ra)
        .collect();
    Some(KeyLight {
        light,
        source: first,
        blended,
        weight,
        left_out,
    })
}

pub use crate::shadows::MAX_SHADOWED_LIGHTS;
pub use og_render_data::clusters::MAX_LIGHTS;

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
    /// How dark the ground takes the shadows of what stands on it, 0 to 1:
    /// the sun's (the cascades) and the share of the map's light they hide
    /// (the ground's height map, `ground_heights.rs`), the ground drawn times the
    /// light it gets with them over the light it would get without them
    /// (Debevec's differential rendering, as Blender's shadow catcher).
    /// 0 draws the ground as photographed.
    pub shadows: f32,
}

impl Default for GroundProjection {
    fn default() -> Self {
        GroundProjection {
            height: 15.0,
            radius: 100.0,
            center: Vec3::ZERO,
            shadows: 1.0,
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
    /// x: -1 for a solid colour, 1 when the background is an environment map drawn from its own
    /// cube at its full size (`map_cube`), y: the blurriness's prefiltered
    /// level (0: sharp), z: the map's background intensity, w: the blended
    /// map's weight.
    pub look: [f32; 4],
    /// (cos, sin) of the map's turn about +y, tone-mapped share, unused;
    /// or RGB for a solid-colour background.
    pub turn: [f32; 4],
    /// xy: box-mip blur biases for the two authored backgrounds;
    /// zw: whether each map has authored box mips instead of GGX levels.
    pub authored: [f32; 4],
    /// x: baked Sky-View LUT altitude (km, negative: unavailable),
    /// y: the frame tick it is valid for. A sliced capture's older
    /// snapshot marches its own sky rather than reading today's LUT.
    pub lut: [f32; 4],
}

/// The map the sky pass draws as the background: its turn, its
/// background intensity and the drawn weight of the map it blends towards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SkyMap {
    pub rotation: f32,
    pub scale: f32,
    pub blend: f32,
    /// Tone-mapped share of the background (0 for an authored LDR sky).
    pub tone_mapped: f32,
    /// Each map's authored box-mip count, or 0 for an HDR/lighting cube.
    pub background_levels: [u32; 2],
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
        if let Background::Color(color) = env.background {
            return SkyParams {
                look: [-1.0, 0.0, 1.0, 0.0],
                turn: color.extend(0.0).to_array(),
                authored: [0.0; 4],
                lut: [-1.0, 0.0, 0.0, 0.0],
            };
        }
        match map {
            Some(m) => SkyParams {
                look: [1.0, level, m.scale, m.blend],
                turn: [m.rotation.cos(), m.rotation.sin(), m.tone_mapped, 0.0],
                authored: [
                    blur * m.background_levels[0].saturating_sub(1) as f32,
                    blur * m.background_levels[1].saturating_sub(1) as f32,
                    u32::from(m.background_levels[0] > 0) as f32,
                    u32::from(m.background_levels[1] > 0) as f32,
                ],
                lut: [-1.0, 0.0, 0.0, 0.0],
            },
            None => SkyParams {
                look: [0.0, level, 1.0, 0.0],
                turn: [1.0, 0.0, 0.0, 0.0],
                authored: [0.0; 4],
                lut: [-1.0, 0.0, 0.0, 0.0],
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
