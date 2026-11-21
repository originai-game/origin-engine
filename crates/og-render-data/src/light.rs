//! The lights the renderer draws and `og bake lightmaps` traces, as data:
//! the directional light and the local lights (point, spot, rect).

use glam::Vec3;

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

/// The sun: one directional light with three cascaded shadow maps. A scene
/// with several (a sun and a moon) draws the one `og_render::key_light` picks.
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
    /// clouds and the weather do (`og_render::Clouds::light_scene`,
    /// `og_render::WeatherLook::light`): its intensity times `beam`, and the
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
    pub fn shadow_softness_and_fill(&self) -> (f32, f32) {
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
    /// a `og_render::RECT_SHADOW_FOV` view from its centre along its normal,
    /// read with a penumbra that widens with the rectangle's size (area.rs).
    /// It fades to zero at `range` as the other lights do.
    Rect {
        half_width: Vec3,
        half_height: Vec3,
        two_sided: bool,
    },
}

/// A point or spot light. Its radiance at distance `d` is
/// `color * intensity / (d² + radius²)`, faded smoothly to zero at `range`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub position: Vec3,
    /// Linear colour.
    pub color: Vec3,
    /// Luminous intensity (candela-like: the radiance at 1 m for radius zero).
    pub intensity: f32,
    /// The distance at which the light reaches zero.
    pub range: f32,
    /// Spherical source radius in metres. Zero preserves a punctual source.
    /// Falloff is softened by radius²; specular uses a representative point.
    pub radius: f32,
    /// Shadows from this light (cube faces or two paraboloid hemispheres for
    /// a point light, `og_render::PointShadows`; one map for a spot). Of the
    /// shadowed lights in view, the `og_render::RenderInfo::shadowed_lights`
    /// (8) with the most radiance at the camera get them; the rest are
    /// counted in
    /// `og_render::FrameStats::shadowed_lights_dropped`.
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
            radius: 0.0,
            cast_shadows: false,
        }
    }
}

/// A spot light's cone (`angle_degrees` its outer half-angle, `penumbra`
/// the fraction it fades over): the cosines of its outer and inner
/// half-angles.
pub fn spot_cone(angle_degrees: f32, penumbra: f32) -> (f32, f32) {
    let outer = angle_degrees.clamp(0.1, 89.0).to_radians();
    let inner = outer * (1.0 - penumbra.clamp(0.0, 1.0));
    (outer.cos(), inner.cos())
}
