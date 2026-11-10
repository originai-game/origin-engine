//! Sprites, text and the overlay drawn over the finished frame.

use crate::{FontHandle, SpriteArray, TextureHandle};
use glam::Vec3;

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

/// What a font's glyph is found by: a character (the font's cmap), or a
/// glyph of the font by its id (a shaped form: a ligature, a joined
/// Arabic letter, a feature's alternate), which no character names. The
/// two never meet: a by-id glyph is drawn only where a [`GlyphPlacement`]
/// asks for it, whatever characters the font maps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GlyphKey {
    Char(char),
    Id(u16),
}

impl From<char> for GlyphKey {
    fn from(c: char) -> GlyphKey {
        GlyphKey::Char(c)
    }
}

impl GlyphKey {
    /// The character, for a glyph found by one.
    pub fn char(self) -> Option<char> {
        match self {
            GlyphKey::Char(c) => Some(c),
            GlyphKey::Id(_) => None,
        }
    }
}

/// One glyph of an MSDF atlas (the layout `msdf-atlas-gen` writes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub key: GlyphKey,
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
    /// The distance field's range in atlas pixels (`msdf-atlas-gen -pxrange`);
    /// 0 for a page of colour glyphs (COLR, cooked as images): drawn as
    /// sprites, untinted but for the text's alpha; their glow, outline and
    /// shadow come from a `silhouette` page.
    pub distance_range: f32,
    /// A page of colour glyphs (distance range 0) that use the text's
    /// colour (COLR palette entry 0xFFFF): each glyph's image is painted
    /// with a black foreground, its twin half the page to the right with a
    /// white one, and the text's colour mixes the two channel by channel
    /// (in sRGB, as the glyph was painted), at the text's alpha.
    pub foreground: bool,
    /// A distance field of colour glyphs' silhouettes (the cook's
    /// `color-field.<n>` and `foreground-field.<n>`): a page drawn before
    /// its colour page that draws their shadows, glow and outline, never a
    /// fill, so the effects lie under the colour images.
    pub silhouette: bool,
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
            let Some(g) = self.glyphs.iter().find(|g| g.key == GlyphKey::Char(ch)) else {
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
    /// A shaped form drawn by the font's glyph id ([`GlyphKey::Id`])
    /// instead of its character's glyph; the character stays in the text
    /// (the cluster's first). `None`: the character's own glyph.
    pub glyph: Option<u16>,
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
