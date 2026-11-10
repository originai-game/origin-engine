//! Scene sprites, text, overlays and tile layers.

use super::*;
pub use og_draw::{
    Glass, Glyph, GlyphKey, GlyphPlacement, Kern, MsdfFont, Overlay, Sprite, Text, TextGlow,
    UI_LAYER, UI_OVERLAY_LAYER, screen_layer,
};

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

/// How a texture is sampled where sprites draw it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    /// Bilinear, with mips when the texture has them.
    #[default]
    Linear,
    /// The nearest texel: crisp pixel art at any scale.
    Nearest,
    /// torcado's cleanEdge (MIT, `third_party/cleanedge/`): pixel art
    /// turned or scaled without blur or jaggies. Each sample reads the 5x5
    /// texels round it and returns one of them, never a blend, chosen by
    /// the edges and 2:1 and 1:1 slopes it finds there, so a turned sprite
    /// keeps its palette and its edges are redrawn at the angle. Texels
    /// outside the sprite's region (a sheet's neighbouring frame) read as
    /// clear. Sprites and tile layers drawn with the sprite shader
    /// (sprite.wgsl) only: texture arrays, the overlay and particles take
    /// it as [`TextureFilter::Nearest`].
    CleanEdge,
}

impl TextureFilter {
    /// Whether the GPU sampler is nearest (both pixel-art filters are).
    pub fn is_nearest(self) -> bool {
        self != TextureFilter::Linear
    }
}

/// A pixel outline round one world sprite ([`RenderScene::sprite_outlines`]),
/// drawn by the sprite shader from the texture's alpha: the clear texels
/// within `width` logical pixels (a diamond, |dx| + |dy| <= width, as a
/// hand-drawn pixel-art line) of an opaque one take `color`. The quad grows
/// by `width` on every side so the line fits outside the art; texels
/// outside the sprite's region read as clear, so a sheet's neighbouring
/// frame never shows. A logical pixel is `pixel` texels of the texture
/// (1: one texel), so a sprite drawn at twice its art's size keeps a line
/// of the style's width rather than doubling it; the test runs at the
/// centre of each logical pixel, so the line is made of whole logical
/// pixels. Untextured sprites take none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteOutline {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    /// The line's width in logical pixels (at most [`MAX_SPRITE_OUTLINE`];
    /// 0 none).
    pub width: f32,
    /// Texels of the texture per logical pixel (0 is 1).
    pub pixel: f32,
    /// Linear RGBA; the alpha times the sprite's opacity.
    pub color: [f32; 4],
}

/// A world sprite drawn with a material's shader
/// ([`RenderScene::sprite_shading`], `sprite_hooks.rs`): the material's
/// shader hooks (of [`crate::ShaderMode::Hooks`] or
/// [`crate::ShaderMode::Sprite`]) and parameters move its corners, change
/// its colour and add light. A material without such a shader, or a
/// sprite drawn from a texture array, draws plain; so does its glow. A
/// faced sprite drawn with one keeps the sprite order (it does not sort
/// with the transparent meshes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteShading {
    /// The index in [`RenderScene::sprites`].
    pub sprite: u32,
    pub material: crate::MaterialHandle,
}

/// The widest [`SpriteOutline::width`], in logical pixels.
pub const MAX_SPRITE_OUTLINE: f32 = 4.0;

/// The texels of an index texture (`index`, RGBA8: alpha the palette index
/// plus one, 0 clear) through row `row` of `palette` (width, height,
/// RGBA8), as [`RenderScene::paletted_texture`] draws them.
pub(crate) fn palette_swap(index: &[u8], palette: (u32, u32, &[u8]), row: u32) -> Vec<u8> {
    let (pw, ph, pal) = palette;
    let row = row.min(ph.saturating_sub(1)) as usize;
    let mut out = Vec::with_capacity(index.len());
    for t in index.as_chunks::<4>().0 {
        let a = t[3] as usize;
        if a == 0 {
            out.extend_from_slice(&[0; 4]);
        } else if a - 1 < pw as usize && ph > 0 {
            let at = (row * pw as usize + a - 1) * 4;
            out.extend_from_slice(&pal[at..at + 4]);
        } else {
            out.extend_from_slice(&[t[0], t[1], t[2], 255]);
        }
    }
    out
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
    /// Its 2D light layers, as [`crate::RenderScene::sprite_light_masks`].
    pub light_mask: u32,
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
