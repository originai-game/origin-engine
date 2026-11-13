//! 2D lights with soft shadows from sprite occluders, and per-sprite effects.

use super::*;

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
    /// How many directions each shadow-casting light's polar shadow map
    /// traces (64..4096).
    pub shadow_angles: u32,
}

impl Default for Lighting2d {
    fn default() -> Self {
        Lighting2d {
            ambient: Vec3::splat(0.15),
            min_layer: i32::MIN,
            max_layer: 99,
            resolution: 0.5,
            shadow_angles: 1024,
        }
    }
}

/// The most different receiver light masks one frame lights apart (a
/// light map each); a further mask is lit as layer 0's (mask 1).
pub const MAX_LIGHT_MASKS_2D: usize = 4;

/// The most 2D lights and occluders one frame draws.
pub const MAX_LIGHTS_2D: usize = 64;
pub const MAX_OCCLUDERS_2D: usize = 128;

/// What shape of light a [`Light2d`] is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Light2dKind {
    /// All around its position, to its radius.
    #[default]
    Point,
    /// A cone along `direction`, full inside `inner_angle` of the axis and
    /// fading to zero at `outer_angle`, to its radius.
    Spot,
    /// Every lit pixel of its layers at `color * intensity`: no radius, no
    /// falloff, no shadows.
    Global,
}

/// A light in the 2D plane (z = 0): `color * intensity` at its centre,
/// falling to zero at `radius` with `(1 - d/r)^falloff`. With
/// `cast_shadows`, occluders between it and a pixel shade the pixel; the
/// penumbra comes from the light's own size `source_radius`. It lights the
/// receivers (sprites, meshes, tile layers) whose light mask shares a bit
/// with `layers`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light2d {
    pub position: Vec2,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
    pub falloff: f32,
    pub source_radius: f32,
    pub cast_shadows: bool,
    pub kind: Light2dKind,
    /// A spot's axis (and a global light's direction), a unit vector.
    pub direction: Vec2,
    /// A spot's half-angles, radians: full inside `inner_angle`, zero past
    /// `outer_angle`.
    pub inner_angle: f32,
    pub outer_angle: f32,
    /// Height above the plane (normal-mapped receivers).
    pub height: f32,
    /// The light layers it lights.
    pub layers: u32,
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
            kind: Light2dKind::Point,
            direction: Vec2::X,
            inner_angle: 0.35,
            outer_angle: 0.6,
            height: 1.0,
            layers: u32::MAX,
        }
    }
}

/// The most vertices an [`Occluder2d`]'s polygon has.
pub const MAX_OCCLUDER_POINTS_2D: usize = 16;

/// A shape that blocks 2D light (walls, crates, characters): a rotated
/// rectangle, usually a sprite's footprint ([`Occluder2d::from_sprite`]),
/// or a polygon. Pixels inside it are not shadowed by it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Occluder2d {
    pub center: Vec2,
    pub half_size: Vec2,
    /// Counter-clockwise, radians.
    pub rotation: f32,
    /// The light layers whose lights it blocks.
    pub layers: u32,
    /// A polygon in place of the rectangle: the first `point_count`
    /// vertices, in order around it (concave is fine), relative to
    /// `center` before `rotation` (already scaled). Fewer than three keep
    /// the rectangle.
    pub points: [Vec2; MAX_OCCLUDER_POINTS_2D],
    pub point_count: u8,
}

impl Default for Occluder2d {
    fn default() -> Self {
        Occluder2d {
            center: Vec2::ZERO,
            half_size: Vec2::splat(0.5),
            rotation: 0.0,
            layers: u32::MAX,
            points: [Vec2::ZERO; MAX_OCCLUDER_POINTS_2D],
            point_count: 0,
        }
    }
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
            ..Default::default()
        }
    }

    /// A polygon occluder: `points` (at most [`MAX_OCCLUDER_POINTS_2D`]
    /// are kept) relative to `center`, turned by `rotation`.
    pub fn polygon(center: Vec2, rotation: f32, points: &[Vec2]) -> Occluder2d {
        let mut o = Occluder2d {
            center,
            rotation,
            ..Default::default()
        };
        let n = points.len().min(MAX_OCCLUDER_POINTS_2D);
        o.points[..n].copy_from_slice(&points[..n]);
        o.point_count = n as u8;
        o
    }

    /// Its outline in the plane, in order around it: the polygon, or the
    /// rectangle's four corners.
    pub fn outline(&self) -> Vec<Vec2> {
        let (s, c) = self.rotation.sin_cos();
        let turn = |p: Vec2| self.center + Vec2::new(p.x * c - p.y * s, p.x * s + p.y * c);
        if self.point_count >= 3 {
            self.points[..self.point_count as usize]
                .iter()
                .map(|&p| turn(p))
                .collect()
        } else {
            let h = self.half_size;
            [
                Vec2::new(-h.x, -h.y),
                Vec2::new(h.x, -h.y),
                Vec2::new(h.x, h.y),
                Vec2::new(-h.x, h.y),
            ]
            .into_iter()
            .map(turn)
            .collect()
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
