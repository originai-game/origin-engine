//! Game effects a scene declares as data: the post stack additions (ambient
//! occlusion, outlines, depth of field, motion blur, god rays, 2D glow,
//! colour grading by LUT, vignette, chromatic aberration, film grain, screen
//! flash), camera shake, decals, 2D lights with soft shadows from sprite
//! occluders, and per-sprite effects. Every one is off by default; a field
//! left at its default costs nothing (no pass is recorded).

use crate::scene::Sprite;
use glam::{Mat4, Vec2, Vec3};
pub use og_render_data::effects::*;

mod decal;
mod grading;
mod two_d;

pub use decal::*;
pub use grading::*;
pub use two_d::*;

// ---------------------------------------------------------------------------
// Post stack additions (fields of `PostEffects`).
// ---------------------------------------------------------------------------

/// Which ambient occlusion algorithm runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AoMethod {
    /// GTAO.
    #[default]
    Auto,
    /// Ground-truth ambient occlusion (horizon search over 3 slices): the
    /// better looking one.
    Gtao,
    /// Hemisphere SSAO with 8 samples: the cheaper one.
    Ssao,
    /// Depth-only disc AO in screen units (three.js-era post SSAO: 14 taps, no normals), fading out from 120 to
    /// 300 m and squared. The radius fields do not apply; `intensity` scales the occlusion.
    Screen,
}

/// The resolution ambient occlusion is searched at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AoResolution {
    /// Every pixel searched, then a 4x4 depth-aware blur.
    #[default]
    Full,
    /// Every second pixel in x and y searched (a quarter of the work) and
    /// blurred at that resolution, then brought to full resolution by a
    /// depth-aware upsample (an `ao-upsample` pass): softer contacts and
    /// thin creases, a fraction of the GPU time.
    Half,
}

/// Screen-space ambient occlusion: darkens the ambient (image-based) light in
/// creases and contact areas. Needs the depth-normal prepass (turned on with it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientOcclusion {
    /// The world-space radius searched for occluders, in metres.
    pub radius: f32,
    /// 0 none, 1 full; above 1 exaggerates.
    pub intensity: f32,
    pub method: AoMethod,
    pub resolution: AoResolution,
    /// How much of the occlusion also darkens direct and local light, 0..1: at 1 a fully occluded pixel keeps 35 %
    /// of it (the look of a deferred SSAO applied to the whole lit colour, as three.js post passes do).
    pub direct: f32,
    /// Above 0, the radius grows with the distance: `min(radius, scale × distance)` metres, a fixed fraction of the
    /// screen up close (three.js-style SSAO, whose radius is set in screen units) instead of a fixed world size that
    /// fills corners near the camera with occlusion.
    pub radius_scale: f32,
}

impl Default for AmbientOcclusion {
    fn default() -> Self {
        AmbientOcclusion {
            radius: 0.6,
            intensity: 1.0,
            method: AoMethod::Auto,
            resolution: AoResolution::Full,
            direct: 0.0,
            radius_scale: 0.0,
        }
    }
}

/// Screen-space outlines from depth and normal edges. Materials with
/// `outlineWidth > 0` get their own colour and width; this sets them for
/// every other opaque surface (`None`: only those materials are outlined).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    /// Linear RGB and coverage.
    pub color: [f32; 4],
    /// Line width in pixels (drawn inside the silhouette).
    pub width: f32,
    /// Relative depth step that counts as a silhouette (0.05 = 5 %).
    pub depth_threshold: f32,
    /// Normal change that counts as a crease (1 - cos of the angle; 0.4 ≈ 53°).
    pub normal_threshold: f32,
}

impl Default for Outline {
    fn default() -> Self {
        Outline {
            color: [0.02, 0.02, 0.03, 1.0],
            width: 2.0,
            depth_threshold: 0.04,
            normal_threshold: 0.4,
        }
    }
}

/// How many taps the depth of field gathers per pixel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DofQuality {
    /// 16 taps (a golden-angle spiral) at half resolution.
    #[default]
    Low,
    /// 48 taps at half resolution: rounder, smoother bokeh.
    High,
}

/// Bokeh depth of field: pixels away from the focus distance blur into
/// discs whose size grows with their distance from the focus band; bright
/// pixels make visible bokeh. Needs the prepass (turned on with it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthOfField {
    /// Distance from the camera that is sharp, in world units.
    pub focus_distance: f32,
    /// Half-width of the sharp band around it.
    pub focus_range: f32,
    /// The largest blur radius, as a fraction of the image height (0.012 ≈ 9 px at 720p).
    pub max_blur: f32,
    /// How quickly blur grows outside the band: full blur `focus_range *
    /// (1 + falloff)` away from the focus distance.
    pub falloff: f32,
    pub quality: DofQuality,
}

impl Default for DepthOfField {
    fn default() -> Self {
        DepthOfField {
            focus_distance: 10.0,
            focus_range: 2.0,
            max_blur: 0.012,
            falloff: 4.0,
            quality: DofQuality::Low,
        }
    }
}

/// Per-object and camera motion blur along the screen-space velocity the
/// prepass writes (turned on with it). Objects move when
/// [`crate::RenderScene::previous_transforms`] is filled; otherwise only the
/// camera's motion blurs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionBlur {
    /// The shutter as a fraction of the frame (0.5 = a 180° shutter).
    pub shutter: f32,
    /// Taps along the velocity (4 to 32).
    pub samples: u32,
    /// Longest blur in pixels.
    pub max_pixels: f32,
}

impl Default for MotionBlur {
    fn default() -> Self {
        MotionBlur {
            shutter: 0.5,
            samples: 12,
            max_pixels: 40.0,
        }
    }
}

/// Screen-space light shafts from the directional light: the sky around the
/// sun blurred radially towards the sun's screen position, masked by
/// geometry. Off while the sun is behind the camera. Needs the prepass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GodRays {
    pub intensity: f32,
    /// How fast the rays fade along their length (0.9..0.99).
    pub decay: f32,
    /// How far towards the sun each ray samples (fraction of the distance).
    pub density: f32,
    /// Linear tint multiplied with the sky colour the rays carry.
    pub tint: Vec3,
    /// Taps per ray (16 to 96).
    pub samples: u32,
    /// Sky brightness (linear, before exposure) below which the sky carries
    /// no rays: only the sun and its glow cast them.
    pub threshold: f32,
}

impl Default for GodRays {
    fn default() -> Self {
        GodRays {
            intensity: 0.5,
            decay: 0.965,
            density: 1.0,
            tint: Vec3::ONE,
            samples: 48,
            threshold: 2.0,
        }
    }
}

/// Lens flare (`post/flare.rs`, `lens_flare.wgsl`): Filament's screen-space
/// flare (after John Chapman), whose ghosts and halo mirror the bright
/// parts of the bloom chain's first level through the centre of the frame
/// with chromatic aberration, blurred and added before the tone curve, the
/// halo under a starburst. The sun and the most important point and spot
/// lights at the camera flare as disc sources of their own radiance too,
/// casting smooth aperture ghosts, each weighted by how much of it the
/// prepass depth shows (Wicked Engine's occlusion grid, on the GPU). Runs after bloom, at a quarter of the frame after TAA, in
/// exposed light, so the exposure and an HDR output apply to it as to bloom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensFlare {
    /// How much of the flare is added (Filament adds its flare at the bloom
    /// strength, 0.1 by default).
    pub intensity: f32,
    /// Ghosts per bright spot, along the line through the centre (0: none).
    pub ghosts: u32,
    /// The ghosts' spacing as a fraction of the way to the centre (0..1).
    pub ghost_spacing: f32,
    /// The exposed brightness a ghost starts above (each tap then clamps at
    /// 1).
    pub ghost_threshold: f32,
    /// The halo's radius as a fraction of the frame's height (0..0.5).
    pub halo_radius: f32,
    /// The halo ring's thickness as a fraction of the height (0: none).
    pub halo_thickness: f32,
    /// The exposed brightness the halo starts above.
    pub halo_threshold: f32,
    /// How far red and blue split along the line to the centre (uv).
    pub chromatic_aberration: f32,
    /// Filament's starburst: the halo modulated by an angular pattern
    /// round the centre that turns with the camera (Filament modulates the
    /// whole flare; its rays drew lines across the ghosts).
    pub starburst: bool,
    /// The sun's own flare (0: none); 1 lets its ghosts reach full strength.
    pub sun: f32,
    /// The point and spot lights' own flares (0: none).
    pub lights: f32,
}

impl Default for LensFlare {
    /// Filament's `BloomOptions` defaults, its strength as the intensity,
    /// but for a fifth of its chromatic aberration and a halo 30 % smaller
    /// (art direction).
    fn default() -> Self {
        LensFlare {
            intensity: 0.1,
            ghosts: 4,
            ghost_spacing: 0.6,
            ghost_threshold: 10.0,
            halo_radius: 0.28,
            halo_thickness: 0.1,
            halo_threshold: 10.0,
            chromatic_aberration: 0.001,
            starburst: true,
            sun: 1.0,
            lights: 1.0,
        }
    }
}

/// Order-independent glow for emissive sprites (2D): sprites with
/// [`SpriteFx::glow`] (and 2D particles with a glow) are also added into a
/// glow buffer, whatever covers them, which the bloom chain blurs and adds
/// back. Works with or without [`crate::Bloom`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    /// How much of the blurred glow is added back.
    pub intensity: f32,
}

impl Default for Glow {
    fn default() -> Self {
        Glow { intensity: 1.0 }
    }
}

/// Darkened (or tinted) edges of the frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vignette {
    /// 0 none, 1 black corners.
    pub intensity: f32,
    /// Where darkening starts, as a fraction of the half-diagonal (0..1).
    pub radius: f32,
    /// Width of the fade (0..1).
    pub smoothness: f32,
    /// Linear colour the edges move towards.
    pub color: Vec3,
}

impl Default for Vignette {
    fn default() -> Self {
        Vignette {
            intensity: 0.35,
            radius: 0.55,
            smoothness: 0.45,
            color: Vec3::ZERO,
        }
    }
}

/// Red and blue split radially towards the edges (lens fringing).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromaticAberration {
    /// Split at the corners, in pixels at 720p (scales with the target).
    pub intensity: f32,
}

impl Default for ChromaticAberration {
    fn default() -> Self {
        ChromaticAberration { intensity: 2.0 }
    }
}

/// Barrel or pincushion distortion of the frame, as a camera lens bends it
/// (RedGPU's lensDistortion, with the radius measured in pixels on any
/// aspect and the frame fitted to its corners): each pixel shows the image
/// at `center + offset × (1 + amount r²) / (1 + max(amount, 0) R²)`, `r`
/// being 1 at the corners of a centred lens and `R` the farthest corner's,
/// so a barrel keeps that corner on the image and shows no black edge
/// wherever its centre is (a pincushion only pulls the image in); beyond
/// the image, with the centre off the frame, is black.
///
/// One pass after the stylisation passes that read the depth (paint, ink
/// and hatching), so their lines bend with the image, and before the
/// quantisation, so a palette stays exact; nearest under a pixel grid.
/// The overlay is not bent. Visual only: what is picked under the
/// pointer is the undistorted frame's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensDistortion {
    /// Above 0 barrel (the centre bulges, straight lines bow outwards: a
    /// wide-angle or fisheye lens), below 0 pincushion (lines bow inwards:
    /// a telephoto), -1..1.
    pub amount: f32,
    /// The lens's centre in fractions of the frame from the top left.
    pub center: Vec2,
}

impl Default for LensDistortion {
    fn default() -> Self {
        LensDistortion {
            amount: 0.15,
            center: Vec2::splat(0.5),
        }
    }
}

/// Which way [`ScreenBlur`] smears the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenBlurKind {
    /// Along the line to the centre: a dash forwards, an impact.
    Zoom,
    /// Round the centre: a spin, a vortex.
    Spin,
    /// Along one direction across the whole frame: a fast pan.
    Directional,
}

/// A blur of the whole lit frame (RedGPU's zoom, radial and directional
/// blurs, with the spin and the directional blur measured in pixels on
/// any aspect): each pixel averages `samples` taps of the image at `t` in
/// -1..1, weighted by a tent for zoom and spin and a Gaussian for
/// directional, at
///
/// - zoom: `p + (center − p) × strength × t` (`strength` the fraction of
///   the way to the centre each way, 0.1 a light dash);
/// - spin: `p` turned about `center` by `strength × t` radians in pixels,
///   so it follows circles on any aspect (0.05 is about 3° each way);
/// - directional: `p + (cos angle, −sin angle) × strength × t` pixels at
///   720 lines (scaled with the target's height), the angle counter-
///   clockwise from the right.
///
/// One HDR pass after motion blur and before the custom passes, bloom and
/// the tonemap, so highlights streak and bloom after it; the overlay is
/// not blurred. Off at strength 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenBlur {
    pub kind: ScreenBlurKind,
    pub strength: f32,
    /// Zoom's and spin's centre, in fractions of the frame from the top
    /// left.
    pub center: Vec2,
    /// Directional's direction (radians, counter-clockwise from the
    /// right).
    pub angle: f32,
    /// Taps per pixel (4 to 64).
    pub samples: u32,
}

impl Default for ScreenBlur {
    fn default() -> Self {
        ScreenBlur {
            kind: ScreenBlurKind::Zoom,
            strength: 0.1,
            center: Vec2::splat(0.5),
            angle: 0.0,
            samples: 16,
        }
    }
}

impl ScreenBlur {
    /// The `blur` and `blur2` rows of fxpost.wgsl's uniform: x the kind (1
    /// zoom, 2 spin, 3 directional; 0 off), y the strength, zw the centre;
    /// x the taps, yz the directional offset in uv at t = 1 for a target
    /// `width` × `height`.
    pub(crate) fn rows(&self, width: f32, height: f32) -> [[f32; 4]; 2] {
        let kind = match self.kind {
            ScreenBlurKind::Zoom => 1.0,
            ScreenBlurKind::Spin => 2.0,
            ScreenBlurKind::Directional => 3.0,
        };
        let px = self.strength * height / 720.0;
        let (sin, cos) = self.angle.sin_cos();
        [
            [kind, self.strength, self.center.x, self.center.y],
            [
                self.samples.clamp(4, 64) as f32,
                cos * px / width.max(1.0),
                -sin * px / height.max(1.0),
                0.0,
            ],
        ]
    }
}

/// Film grain: luminance-weighted noise, a new pattern every
/// [`crate::RenderScene::tick`] (so a given tick always has the same grain).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilmGrain {
    /// Noise amplitude in display units (0.04 is subtle).
    pub intensity: f32,
    /// Grain size in pixels.
    pub size: f32,
}

impl Default for FilmGrain {
    fn default() -> Self {
        FilmGrain {
            intensity: 0.04,
            size: 1.5,
        }
    }
}

/// A full-screen flash (a hit, a pickup): the display colour mixed towards
/// `color` by `amount`. The game animates `amount` per tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenFlash {
    /// Linear display colour.
    pub color: Vec3,
    /// 0 none, 1 the flat colour.
    pub amount: f32,
}

impl Default for ScreenFlash {
    fn default() -> Self {
        ScreenFlash {
            color: Vec3::ONE,
            amount: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Camera shake.
// ---------------------------------------------------------------------------

/// A screen-shake offset applied to the camera (not to the image, so no
/// edges show): the view moves by `offset` in view space, as a fraction of
/// the view height at the camera's target, and rolls by `roll_degrees`.
/// Visual only. [`CameraShake::trauma`] makes a deterministic shake from a
/// trauma value and the tick; hit-stop is the game's (it holds the tick it
/// passes to particles and trails).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraShake {
    pub offset: Vec2,
    pub roll_degrees: f32,
}

impl CameraShake {
    /// The shake for `trauma` (0..1, squared for the amplitude, as in
    /// "Math for Game Programmers: Juicing Your Cameras") at `tick`: smooth
    /// value noise at `frequency` Hz (`tick_rate` ticks per second), seeded,
    /// up to `max_offset` (fraction of the view height) and `max_roll_degrees`.
    pub fn trauma(
        trauma: f32,
        tick: u64,
        tick_rate: f32,
        seed: u64,
        max_offset: f32,
        max_roll_degrees: f32,
        frequency: f32,
    ) -> CameraShake {
        let amount = trauma.clamp(0.0, 1.0).powi(2);
        if amount == 0.0 {
            return CameraShake::default();
        }
        let t = tick as f32 / tick_rate.max(1e-3) * frequency.max(0.0);
        let s = crate::particles::hash_seed(seed);
        let n = |stream: u32| crate::particles::noise1(t, s ^ stream.wrapping_mul(0x9e37_79b9));
        CameraShake {
            offset: Vec2::new(n(1), n(2)) * max_offset * amount,
            roll_degrees: n(3) * max_roll_degrees * amount,
        }
    }
}
