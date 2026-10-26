//! Game effects a scene declares as data: the post stack additions (ambient
//! occlusion, outlines, depth of field, motion blur, god rays, 2D glow,
//! colour grading by LUT, vignette, chromatic aberration, film grain, screen
//! flash), camera shake, decals, 2D lights with soft shadows from sprite
//! occluders, and per-sprite effects. Every one is off by default; a field
//! left at its default costs nothing (no pass is recorded).

use crate::scene::Sprite;
use glam::{Mat4, Vec2, Vec3};

// ---------------------------------------------------------------------------
// Post stack additions (fields of `PostEffects`).
// ---------------------------------------------------------------------------

/// Which ambient occlusion algorithm runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AoMethod {
    /// GTAO on the full tier (WebGPU, Metal, Vulkan), SSAO on the
    /// no-compute tier (WebGL2).
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

/// Colour grading through a 3D lookup table, applied to the display colour
/// (after the tonemap).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorGrading {
    pub lut: LutHandle,
    /// 0 leaves the image as tonemapped, 1 applies the LUT fully.
    pub intensity: f32,
}

/// A photo editor's colour adjustments of the display colour, after the
/// tone curve and [`ColorGrading`]'s table, in the order of the fields:
/// white balance, brightness and contrast, colour balance, hue,
/// saturation, vibrance, greyscale, invert and threshold. Every field at
/// zero leaves the image unchanged (`preserve_luminosity` acts only with a
/// colour balance). Each acts on the sRGB-encoded display colour (the
/// white balance on its linear light) and clamps it to 0..1.
///
/// They cost no pass: the renderer bakes them, after the grading table at
/// its intensity, into one 32³ table on the CPU ([`ColorAdjust::apply`] of
/// each of its 32,768 colours) whenever either changes, and the tonemap
/// samples that table instead. A frame whose adjustments hold still costs
/// what grading costs; one that changes them pays the bake: about 0.4 ms
/// with one adjustment on, 3.5 ms with all of them and 4.3 ms with a
/// grading table too (release build, one core of a 2.6 GHz Xeon).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorAdjust {
    /// White balance: towards warm (orange, positive) or cool (blue,
    /// negative), -1..1; Unity's Temperature divided by 100 (a von Kries
    /// scaling in LMS from a white point moved along the daylight locus).
    pub temperature: f32,
    /// White balance: towards magenta (positive) or green (negative),
    /// -1..1, as Lightroom's and Unity's Tint (divided by 100).
    pub tint: f32,
    /// Added to every channel, -1..1.
    pub brightness: f32,
    /// Around mid grey: positive spreads the tones by `1 / (1 - contrast)`,
    /// negative pulls them in by `1 + contrast` (-1 is flat grey).
    pub contrast: f32,
    /// The colour balance of the shadows (luminance below 0.5, fading out
    /// towards it): shifts along cyan-red (x), magenta-green (y) and
    /// yellow-blue (z), -1..1, added to red, green and blue.
    pub shadows: Vec3,
    /// The colour balance of the mid-tones (what neither of the others takes).
    pub midtones: Vec3,
    /// The colour balance of the highlights (luminance above 0.5, fading in from it).
    pub highlights: Vec3,
    /// Whether the colour balance keeps each pixel's luminance, so it moves
    /// only the hue (Photoshop's Preserve Luminosity).
    pub preserve_luminosity: bool,
    /// Hue rotation about the grey axis, in degrees.
    pub hue: f32,
    /// -1 grey, 0 unchanged, 1 twice the colour (the chroma around the
    /// luminance scaled by `1 + saturation`).
    pub saturation: f32,
    /// Saturation that spares colours already saturated and skin tones,
    /// -1..1.
    pub vibrance: f32,
    /// How far towards its luminance (grey) the colour goes, 0..1.
    pub grayscale: f32,
    /// How far towards its complement (`1 - c`) the colour goes, 0..1.
    pub invert: f32,
    /// Above 0, each pixel white where its luminance reaches this level
    /// and black below it (0..1); 0 is off. The table's 32 steps soften
    /// the edge over about 1/31 of the range.
    pub threshold: f32,
}

impl Default for ColorAdjust {
    fn default() -> Self {
        ColorAdjust {
            temperature: 0.0,
            tint: 0.0,
            brightness: 0.0,
            contrast: 0.0,
            shadows: Vec3::ZERO,
            midtones: Vec3::ZERO,
            highlights: Vec3::ZERO,
            preserve_luminosity: true,
            hue: 0.0,
            saturation: 0.0,
            vibrance: 0.0,
            grayscale: 0.0,
            invert: 0.0,
            threshold: 0.0,
        }
    }
}

/// Rec. 709 luma weights (`LUMA` in fullscreen.wgsl).
const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

/// The sRGB transfer curves as tables of 4096 steps read with linear
/// interpolation: within 2e-5 of the curves (well inside an 8-bit step)
/// and much cheaper than `powf`, for the 32,768 colours of a bake.
struct SrgbTables {
    /// Linear light by the encoded value.
    decode: Vec<f32>,
    /// The encoded value by linear light.
    encode: Vec<f32>,
}

const SRGB_STEPS: usize = 4096;

fn srgb_tables() -> &'static SrgbTables {
    static TABLES: std::sync::OnceLock<SrgbTables> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let at = |i: usize| i as f32 / SRGB_STEPS as f32;
        SrgbTables {
            decode: (0..=SRGB_STEPS)
                .map(|i| {
                    let v = at(i);
                    if v <= 0.04045 {
                        v / 12.92
                    } else {
                        ((v + 0.055) / 1.055).powf(2.4)
                    }
                })
                .collect(),
            encode: (0..=SRGB_STEPS)
                .map(|i| {
                    let v = at(i);
                    if v <= 0.003_130_8 {
                        v * 12.92
                    } else {
                        1.055 * v.powf(1.0 / 2.4) - 0.055
                    }
                })
                .collect(),
        }
    })
}

/// `table` at `v` (0..1, clamped), interpolated.
fn srgb_lookup(table: &[f32], v: f32) -> f32 {
    let p = v.clamp(0.0, 1.0) * SRGB_STEPS as f32;
    let i = (p as usize).min(SRGB_STEPS - 1);
    table[i] + (table[i + 1] - table[i]) * (p - i as f32)
}

fn srgb_decode(c: Vec3) -> Vec3 {
    let t = &srgb_tables().decode;
    Vec3::new(
        srgb_lookup(t, c.x),
        srgb_lookup(t, c.y),
        srgb_lookup(t, c.z),
    )
}

fn srgb_encode(c: Vec3) -> Vec3 {
    let t = &srgb_tables().encode;
    Vec3::new(
        srgb_lookup(t, c.x),
        srgb_lookup(t, c.y),
        srgb_lookup(t, c.z),
    )
}

/// Linear sRGB to LMS cone response (Unity's `LIN_2_LMS_MAT`, CAT02 over
/// the sRGB primaries), in columns.
const LIN_TO_LMS: glam::Mat3 = glam::Mat3::from_cols(
    Vec3::new(3.90405e-1, 7.08416e-2, 2.31082e-2),
    Vec3::new(5.49941e-1, 9.63172e-1, 1.28021e-1),
    Vec3::new(8.92632e-3, 1.35775e-3, 9.36245e-1),
);

/// The LMS scale that maps the white of `temperature` and `tint` (-1..1)
/// to D65 (Unity's `ColorBalanceToLMSCoeffs`): the reference white moved
/// along x on the daylight locus (`StandardIlluminantY`), then along y.
fn white_balance(temperature: f32, tint: f32) -> Vec3 {
    let t1 = temperature * 100.0 / 60.0;
    let t2 = tint * 100.0 / 60.0;
    let x = 0.31271 - t1 * if t1 < 0.0 { 0.1 } else { 0.05 };
    let y = 2.87 * x - 3.0 * x * x - 0.275_095_07 + t2 * 0.05;
    let (cx, cz) = (x / y, (1.0 - x - y) / y);
    let lms = Vec3::new(
        0.7328 * cx + 0.4296 - 0.1624 * cz,
        -0.7036 * cx + 1.6975 + 0.0061 * cz,
        0.0030 * cx + 0.0136 + 0.9834 * cz,
    );
    Vec3::new(0.949_237, 1.035_42, 1.087_28) / lms
}

impl ColorAdjust {
    /// Whether it changes nothing (every adjustment at zero).
    pub fn is_identity(&self) -> bool {
        let ColorAdjust {
            temperature,
            tint,
            brightness,
            contrast,
            shadows,
            midtones,
            highlights,
            preserve_luminosity: _,
            hue,
            saturation,
            vibrance,
            grayscale,
            invert,
            threshold,
        } = *self;
        [
            temperature,
            tint,
            brightness,
            contrast,
            hue,
            saturation,
            vibrance,
            grayscale,
            invert,
            threshold,
        ]
        .iter()
        .all(|&v| v == 0.0)
            && shadows == Vec3::ZERO
            && midtones == Vec3::ZERO
            && highlights == Vec3::ZERO
    }

    /// The adjusted display colour of `c` (sRGB-encoded, 0..1).
    pub fn apply(&self, c: Vec3) -> Vec3 {
        self.prepare()(c)
    }

    /// [`ColorAdjust::apply`] with what it computes once per table (the
    /// white balance and hue rotation as matrices) computed now.
    pub(crate) fn prepare(&self) -> impl Fn(Vec3) -> Vec3 + '_ {
        let unit = |c: Vec3| c.clamp(Vec3::ZERO, Vec3::ONE);
        let towards_luma = move |c: Vec3, k: f32| {
            let l = c.dot(LUMA);
            unit(Vec3::splat(l) + (c - l) * k)
        };
        // The white balance in linear light: to LMS, scaled, back.
        let balance = (self.temperature != 0.0 || self.tint != 0.0).then(|| {
            LIN_TO_LMS.inverse()
                * glam::Mat3::from_diagonal(white_balance(self.temperature, self.tint))
                * LIN_TO_LMS
        });
        let contrast = self.contrast.clamp(-1.0, 0.999);
        let slope = if contrast > 0.0 {
            1.0 / (1.0 - contrast)
        } else {
            1.0 + contrast
        };
        let shifts = self.shadows != Vec3::ZERO
            || self.midtones != Vec3::ZERO
            || self.highlights != Vec3::ZERO;
        // Rotation about the grey axis (RedGPU's hueSaturation, CSS
        // hue-rotate's axis): rows w, w.zxy, w.yzx.
        let hue = (self.hue != 0.0).then(|| {
            let (s, k) = self.hue.to_radians().sin_cos();
            let r3 = 3f32.sqrt();
            let w = (Vec3::new(2.0 * k, -r3 * s - k, r3 * s - k) + 1.0) / 3.0;
            glam::Mat3::from_cols(w, Vec3::new(w.z, w.x, w.y), Vec3::new(w.y, w.z, w.x)).transpose()
        });
        move |c: Vec3| {
            let mut c = unit(c);
            if let Some(m) = balance {
                c = srgb_encode(unit(m * srgb_decode(c)));
            }
            if self.contrast != 0.0 || self.brightness != 0.0 {
                c = unit((c - 0.5) * slope + 0.5 + self.brightness);
            }
            if shifts {
                let l = c.dot(LUMA);
                let smooth = |e0: f32, e1: f32, x: f32| {
                    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                };
                let shadow = 1.0 - smooth(0.0, 0.5, l);
                let highlight = smooth(0.5, 1.0, l);
                let mid = 1.0 - shadow - highlight;
                let shifted =
                    c + self.shadows * shadow + self.midtones * mid + self.highlights * highlight;
                let after = shifted.dot(LUMA);
                c = unit(if self.preserve_luminosity && after > 0.0 {
                    shifted * (l / after)
                } else {
                    shifted
                });
            }
            if let Some(m) = hue {
                c = unit(m * c);
            }
            if self.saturation != 0.0 {
                c = towards_luma(c, 1.0 + self.saturation.max(-1.0));
            }
            if self.vibrance != 0.0 {
                // RedGPU's vibrance: weaker where the colour is already
                // saturated (a logistic falloff past 0.6) and on skin
                // tones (red over green over blue).
                let spread = c.max_element() - c.min_element();
                let protect = 1.0 / (1.0 + (6.0 * (spread - 0.6)).exp());
                let skin = if c.x > c.y && c.y > c.z {
                    let t = (((c.x - c.z) / c.x.max(1e-3) - 0.3) / 0.5).clamp(0.0, 1.0);
                    1.0 - t * t * (3.0 - 2.0 * t) * 0.7
                } else {
                    1.0
                };
                c = towards_luma(c, 1.0 + self.vibrance * protect * skin);
            }
            if self.grayscale != 0.0 {
                c = c.lerp(Vec3::splat(c.dot(LUMA)), self.grayscale.clamp(0.0, 1.0));
            }
            if self.invert != 0.0 {
                c = c.lerp(Vec3::ONE - c, self.invert.clamp(0.0, 1.0));
            }
            if self.threshold > 0.0 {
                c = Vec3::splat(if c.dot(LUMA) >= self.threshold {
                    1.0
                } else {
                    0.0
                });
            }
            unit(c)
        }
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

// ---------------------------------------------------------------------------
// Colour lookup tables.
// ---------------------------------------------------------------------------

pub use crate::scene::LutHandle;

/// The LUT side every renderer expects.
pub const LUT_SIZE: u32 = 32;

/// A 32³ colour lookup table: `rgba` holds `size³` RGBA8 texels, red
/// fastest, then green, then blue (the `.cube` order), mapping a display
/// (sRGB-encoded) colour to its graded display colour.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorLut {
    pub size: u32,
    pub rgba: Vec<u8>,
}

impl ColorLut {
    /// The identity table: grading with it changes nothing (within the LUT's
    /// 8-bit precision).
    pub fn identity() -> ColorLut {
        ColorLut::from_fn(|c| c)
    }

    /// A table made by `f(display rgb in 0..1) -> display rgb`.
    pub fn from_fn(f: impl Fn(Vec3) -> Vec3) -> ColorLut {
        let n = LUT_SIZE;
        let mut rgba = Vec::with_capacity((n * n * n * 4) as usize);
        let max = (n - 1) as f32;
        for b in 0..n {
            for g in 0..n {
                for r in 0..n {
                    let c = f(Vec3::new(r as f32 / max, g as f32 / max, b as f32 / max));
                    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    rgba.extend_from_slice(&[q(c.x), q(c.y), q(c.z), 255]);
                }
            }
        }
        ColorLut { size: n, rgba }
    }

    /// Parses an Adobe/Resolve `.cube` file with `LUT_3D_SIZE 32` (other
    /// sizes are resampled to 32 with trilinear filtering). `DOMAIN_MIN` and
    /// `DOMAIN_MAX` must be 0 and 1.
    pub fn from_cube(text: &str) -> Result<ColorLut, String> {
        let mut size = 0u32;
        let mut values: Vec<Vec3> = Vec::new();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("TITLE") {
                continue;
            }
            let mut parts = line.split_whitespace();
            let first = parts.next().unwrap_or("");
            match first {
                "LUT_3D_SIZE" => {
                    size = parts
                        .next()
                        .and_then(|v| v.parse().ok())
                        .ok_or_else(|| format!("line {}: bad LUT_3D_SIZE", line_no + 1))?;
                }
                "LUT_1D_SIZE" => return Err("a 1D .cube LUT is not a 3D colour table".into()),
                "DOMAIN_MIN" | "DOMAIN_MAX" => {
                    let want = if first == "DOMAIN_MIN" { 0.0 } else { 1.0 };
                    let ok = parts.all(|v| v.parse::<f32>().is_ok_and(|x| x == want));
                    if !ok {
                        return Err(format!("line {}: {first} must be {want}", line_no + 1));
                    }
                }
                _ => {
                    let nums: Result<Vec<f32>, _> =
                        line.split_whitespace().map(str::parse::<f32>).collect();
                    match nums {
                        Ok(v) if v.len() == 3 => values.push(Vec3::new(v[0], v[1], v[2])),
                        _ => return Err(format!("line {}: '{line}'", line_no + 1)),
                    }
                }
            }
        }
        if size < 2 || values.len() != (size * size * size) as usize {
            return Err(format!(
                "LUT_3D_SIZE {size} needs {} entries, found {}",
                size * size * size,
                values.len()
            ));
        }
        let at = |r: u32, g: u32, b: u32| values[(r + g * size + b * size * size) as usize];
        Ok(ColorLut::from_fn(|c| {
            let p = c * (size - 1) as f32;
            let i = p.floor().min(Vec3::splat((size - 2) as f32));
            let f = p - i;
            let (r, g, b) = (i.x as u32, i.y as u32, i.z as u32);
            let lerp = |a: Vec3, b: Vec3, t: f32| a + (b - a) * t;
            let c00 = lerp(at(r, g, b), at(r + 1, g, b), f.x);
            let c10 = lerp(at(r, g + 1, b), at(r + 1, g + 1, b), f.x);
            let c01 = lerp(at(r, g, b + 1), at(r + 1, g, b + 1), f.x);
            let c11 = lerp(at(r, g + 1, b + 1), at(r + 1, g + 1, b + 1), f.x);
            lerp(lerp(c00, c10, f.y), lerp(c01, c11, f.y), f.z)
        }))
    }

    /// Reads a LUT laid out as a horizontal strip of `size` slices (the
    /// Unity/Unreal 1024x32 PNG): slice `b` holds red along x, green along y
    /// (row 0 is green 0). `rgba` is the strip's RGBA8 pixels, rows top to bottom.
    pub fn from_strip(width: u32, height: u32, rgba: &[u8]) -> Result<ColorLut, String> {
        if height == 0 || width != height * height || rgba.len() != (width * height * 4) as usize {
            return Err(format!(
                "a LUT strip is (n*n)xn RGBA8 pixels; got {width}x{height} with {} bytes",
                rgba.len()
            ));
        }
        let n = height;
        let at = |r: u32, g: u32, b: u32| {
            let i = ((g * width + b * n + r) * 4) as usize;
            Vec3::new(rgba[i] as f32, rgba[i + 1] as f32, rgba[i + 2] as f32) / 255.0
        };
        if n == LUT_SIZE {
            let mut out = Vec::with_capacity(rgba.len());
            for b in 0..n {
                for g in 0..n {
                    for r in 0..n {
                        let i = ((g * width + b * n + r) * 4) as usize;
                        out.extend_from_slice(&[rgba[i], rgba[i + 1], rgba[i + 2], 255]);
                    }
                }
            }
            return Ok(ColorLut { size: n, rgba: out });
        }
        Ok(ColorLut::from_fn(|c| {
            let p = (c * (n - 1) as f32).round();
            at(p.x as u32, p.y as u32, p.z as u32)
        }))
    }

    /// The table the tonemap samples for `adjust` after `grade` (a
    /// [`LUT_SIZE`] table and its intensity, mixed in linear light as the
    /// tonemap mixes it). The grid points fall on the grading table's
    /// texels, so it is read without filtering.
    pub(crate) fn adjusted(adjust: &ColorAdjust, grade: Option<(&ColorLut, f32)>) -> ColorLut {
        let apply = adjust.prepare();
        let n = LUT_SIZE as usize;
        let decode = &srgb_tables().decode;
        // The grid's steps, encoded and in linear light, and the linear
        // light of each texel value.
        let steps: Vec<f32> = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
        let linear: Vec<f32> = steps.iter().map(|&v| srgb_lookup(decode, v)).collect();
        let texel: Vec<f32> = (0..=255)
            .map(|v| srgb_lookup(decode, v as f32 / 255.0))
            .collect();
        let grade = grade.map(|(lut, k)| {
            debug_assert_eq!(lut.size, LUT_SIZE);
            (lut, k.clamp(0.0, 1.0))
        });
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        let mut rgba = vec![0u8; n * n * n * 4];
        for (i, out) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (r, g, b) = (i % n, i / n % n, i / (n * n));
            let c = match grade {
                Some((lut, k)) => {
                    let t = &lut.rgba[i * 4..i * 4 + 3];
                    let graded = Vec3::new(
                        texel[t[0] as usize],
                        texel[t[1] as usize],
                        texel[t[2] as usize],
                    );
                    srgb_encode(Vec3::new(linear[r], linear[g], linear[b]).lerp(graded, k))
                }
                None => Vec3::new(steps[r], steps[g], steps[b]),
            };
            let c = apply(c);
            *out = [q(c.x), q(c.y), q(c.z), 255];
        }
        ColorLut {
            size: LUT_SIZE,
            rgba,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let n = self.size as usize;
        if self.size != LUT_SIZE || self.rgba.len() != n * n * n * 4 {
            return Err(format!(
                "a colour LUT is {LUT_SIZE}³ RGBA8 ({} bytes); got size {} with {} bytes",
                LUT_SIZE.pow(3) * 4,
                self.size,
                self.rgba.len()
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

/// The most decals one frame draws (the rest are counted in
/// [`crate::FrameStats::decals_dropped`]).
pub const MAX_DECALS: usize = 32;

/// A projected decal (scorch marks, blood, posters, bullet holes): a unit
/// box (-0.5..0.5 on each axis) placed by `transform`, projected along its
/// local -z onto every lit opaque surface inside it, in the forward pass
/// before lighting, so the decal is shaded with the surface. Its image is
/// `uv_rect` of [`crate::RenderScene::decal_atlas`] (or a solid colour
/// without an atlas).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decal {
    pub transform: Mat4,
    /// `[u0, v0, u1, v1]` in the atlas, v down (local +x is u, +y is up).
    pub uv_rect: [f32; 4],
    /// Linear RGBA multiplied with the atlas; alpha is the opacity.
    pub color: [f32; 4],
    /// Linear emitted light added where the decal covers (glowing runes, lava).
    pub emissive: Vec3,
    /// Surface roughness under the decal (negative: keep the surface's).
    pub roughness: f32,
    /// Surfaces facing away from the projection direction by more than this
    /// angle (degrees) are not covered; the last 15° fade.
    pub angle_fade_degrees: f32,
}

impl Default for Decal {
    fn default() -> Self {
        Decal {
            transform: Mat4::IDENTITY,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: [1.0; 4],
            emissive: Vec3::ZERO,
            roughness: -1.0,
            angle_fade_degrees: 75.0,
        }
    }
}

impl Decal {
    /// A decal lying on a surface at `position` facing along `normal`
    /// (projected against it), `size` wide and long, `depth` thick, turned by
    /// `rotation` radians about the normal.
    pub fn on_surface(
        position: Vec3,
        normal: Vec3,
        size: Vec2,
        depth: f32,
        rotation: f32,
    ) -> Decal {
        let z = normal.normalize_or(Vec3::Y);
        let helper = if z.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
        let x = helper.cross(z).normalize();
        let y = z.cross(x);
        let basis = Mat4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            position.extend(1.0),
        );
        Decal {
            transform: basis
                * Mat4::from_rotation_z(rotation)
                * Mat4::from_scale(Vec3::new(size.x, size.y, depth)),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// 2D lighting.
// ---------------------------------------------------------------------------

/// The 2D lighting of the sprite layers `min_layer..=max_layer`: their
/// colour (sprites, 2D particles and the tile layers there whose
/// [`crate::TileLayer::lit`] is set) is multiplied by `ambient` plus every
/// [`Light2d`] reaching the pixel, shadowed by [`Occluder2d`]s. Other
/// layers (UI) are drawn as authored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lighting2d {
    /// Linear light everywhere (0 is darkness outside the lights).
    pub ambient: Vec3,
    pub min_layer: i32,
    pub max_layer: i32,
    /// The light map's resolution as a fraction of the target (0.25..1).
    pub resolution: f32,
}

impl Default for Lighting2d {
    fn default() -> Self {
        Lighting2d {
            ambient: Vec3::splat(0.15),
            min_layer: i32::MIN,
            max_layer: 99,
            resolution: 0.5,
        }
    }
}

/// The most 2D lights and occluders one frame draws.
pub const MAX_LIGHTS_2D: usize = 64;
pub const MAX_OCCLUDERS_2D: usize = 128;

/// A light in the 2D plane (z = 0): `color * intensity` at its centre,
/// falling to zero at `radius` with `(1 - d/r)^falloff`. With
/// `cast_shadows`, occluders between it and a pixel shade the pixel; the
/// penumbra comes from the light's own size `source_radius`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light2d {
    pub position: Vec2,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub source_radius: f32,
    pub cast_shadows: bool,
}

impl Default for Light2d {
    fn default() -> Self {
        Light2d {
            position: Vec2::ZERO,
            color: Vec3::ONE,
            intensity: 1.0,
            radius: 5.0,
            falloff: 2.0,
            source_radius: 0.25,
            cast_shadows: true,
        }
    }
}

/// A rotated rectangle that blocks 2D light (walls, crates, characters):
/// usually a sprite's footprint ([`Occluder2d::from_sprite`]). Pixels
/// inside it are not shadowed by it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Occluder2d {
    pub center: Vec2,
    pub half_size: Vec2,
    /// Counter-clockwise, radians.
    pub rotation: f32,
}

impl Occluder2d {
    /// The rectangle a sprite covers (its size, anchor and rotation), shrunk
    /// by `inset` on each side (sprites with soft edges cast from their body).
    pub fn from_sprite(sprite: &Sprite, inset: f32) -> Occluder2d {
        let (s, c) = sprite.rotation.sin_cos();
        let local = (Vec2::splat(0.5) - sprite.anchor) * sprite.size;
        let center = sprite.position.truncate()
            + Vec2::new(local.x * c - local.y * s, local.x * s + local.y * c);
        Occluder2d {
            center,
            half_size: (sprite.size * 0.5 - Vec2::splat(inset)).max(Vec2::ZERO),
            rotation: sprite.rotation,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-sprite effects.
// ---------------------------------------------------------------------------

/// How a sprite (or 2D particle) is blended over what is below it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpriteBlend {
    /// Ordinary alpha blending.
    #[default]
    Alpha,
    /// Added (light, fire, magic): order-independent within a layer.
    Additive,
}

/// Effects on one sprite of [`crate::RenderScene::sprites`] (by index):
/// a hit flash, a glow, the blend and whether it is tone mapped. Sprites
/// without an entry draw as before.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteFx {
    /// The index of the sprite in [`crate::RenderScene::sprites`].
    pub sprite: u32,
    /// Linear RGB the sprite's colour mixes towards, and how far (w, 0..1),
    /// keeping its alpha: a white hit flash is `[1, 1, 1, 1]`.
    pub flash: [f32; 4],
    /// Strength of its contribution to the order-independent glow buffer
    /// ([`Glow`]); 0 none.
    pub glow: f32,
    pub blend: SpriteBlend,
    /// three.js `Material.toneMapped`: true (the default) draws the sprite
    /// into the lit scene, so exposure, bloom and the tone curve carry it
    /// like any surface. False keeps its colours as given: it is drawn in
    /// place, depth tested and blended as before, but the tonemap shows its
    /// pixels unexposed and uncurved (clamped to 0..1) and bloom neither
    /// gathers from them nor spills onto them; grading, vignette, grain and
    /// the screen flash after the curve still apply (README "Untonemapped
    /// sprites"). An alpha-blended sprite only: an additive one adds light
    /// to the scene and is always tone mapped. No fog either.
    pub tone_mapped: bool,
}

impl Default for SpriteFx {
    fn default() -> Self {
        SpriteFx {
            sprite: 0,
            flash: [1.0, 1.0, 1.0, 0.0],
            glow: 0.0,
            blend: SpriteBlend::Alpha,
            tone_mapped: true,
        }
    }
}
