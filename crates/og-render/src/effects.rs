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
}

impl Default for AmbientOcclusion {
    fn default() -> Self {
        AmbientOcclusion {
            radius: 0.6,
            intensity: 1.0,
            method: AoMethod::Auto,
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
