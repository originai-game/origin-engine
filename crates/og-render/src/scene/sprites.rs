//! Scene sprites, text, overlays and tile layers.

use super::*;

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

impl Sprite {
    /// The same quad with its anchor point moved to `pivot` (x and y; the
    /// sprite keeps its z): the anchor is shifted by the offset in the
    /// sprite's own rotated frame, so every corner stays where it was. A
    /// faced sprite turns about its anchor, so the pieces of one faced
    /// nine-slice ([`crate::nine_slice`]) are anchored at the panel's anchor to
    /// turn as one panel. An axis of zero size keeps its anchor.
    pub fn anchored_at(&self, pivot: Vec3) -> Sprite {
        let d = pivot.truncate() - self.position.truncate();
        let (sin, cos) = self.rotation.sin_cos();
        // The offset in the quad's frame (the rotation undone).
        let local = glam::Vec2::new(d.x * cos + d.y * sin, -d.x * sin + d.y * cos);
        let shift = |l: f32, size: f32| if size != 0.0 { l / size } else { 0.0 };
        Sprite {
            position: Vec3::new(pivot.x, pivot.y, self.position.z),
            anchor: self.anchor
                + glam::Vec2::new(shift(local.x, self.size.x), shift(local.y, self.size.y)),
            ..*self
        }
    }
}

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
/// MSDF fonts. Drawn on every backend; included in [`crate::Renderer::capture`]
/// and counted in [`crate::FrameStats`].
#[derive(Clone, Debug, PartialEq)]
pub struct Overlay {
    pub sprites: Vec<Sprite>,
    /// Overlay sprites drawn from a layer of a texture array instead of
    /// their texture ([`SpriteArray::sprite`] indexes [`Overlay::sprites`]):
    /// one draw per array whichever layer each shows, as in the world.
    pub sprite_arrays: Vec<SpriteArray>,
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
    /// The UI pass's shapes, clip shapes, layers, text effects and vectors
    /// ([`crate::ui`]), sorted with the sprites and text by layer.
    pub ui: crate::ui::UiList,
}

/// The UI's layer among screen sprites and screen text (the IDL's
/// `UI_LAYER`): a game's overlay content at or below it draws under every
/// UI element, above it over every one (a HUD element of sprites on a UI
/// panel), in its layers' order. The overlay itself sorts by layer only;
/// og-runtime places the UI at [`UI_OVERLAY_LAYER`] and a game's screen
/// layers by [`screen_layer`].
pub const UI_LAYER: i32 = 1000;

/// The overlay layer the UI's painter level 0 draws on (level n on this plus
/// n): above every screen layer up to [`UI_LAYER`].
pub const UI_OVERLAY_LAYER: i32 = 1 << 20;

/// The overlay layer of a game's screen sprite or screen text of `layer`:
/// its own at or below [`UI_LAYER`], under the UI; above it, lifted past
/// every painter level the UI can use (2^20 of them), the order among
/// screen layers kept.
pub fn screen_layer(layer: i32) -> i32 {
    if layer > UI_LAYER {
        layer.saturating_add(1 << 21)
    } else {
        layer
    }
}

impl Overlay {
    /// An empty overlay of `width`×`height` CSS pixels at `scale` device
    /// pixels per CSS pixel.
    pub fn new(width: f32, height: f32, scale: f32) -> Overlay {
        Overlay {
            sprites: Vec::new(),
            sprite_arrays: Vec::new(),
            texts: Vec::new(),
            glass: Vec::new(),
            width,
            height,
            scale,
            clips: Vec::new(),
            ui: crate::ui::UiList::default(),
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
