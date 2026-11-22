//! Sprites, text and the overlay drawn over the finished frame.

use crate::glyphs::GlyphTier;
use crate::{SpriteArray, TextureHandle};
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

/// One glyph of a [`Text`]: its raster in the glyph atlas
/// ([`crate::glyphs`]) and where its quad lies.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextGlyph {
    /// The quad from the pen at the first line's baseline, in em, y up:
    /// `[left, bottom, right, top]`.
    pub rect: [f32; 4],
    /// Its raster's tier: a distance field ([`GlyphTier::Field`]) or a
    /// colour glyph ([`GlyphTier::Color`]).
    pub tier: GlyphTier,
    /// Its page of that tier.
    pub page: u16,
    /// Its rectangle in the page, texels: `[left, top, right, bottom]`.
    pub uv: [u16; 4],
}

/// Text in the world or the overlay: glyphs laid out and aligned by its
/// writer (og-text) and rastered into the glyph atlas
/// ([`crate::glyphs::Atlas`]), each drawn from its distance field, or a
/// colour glyph in its own colours. The first line's baseline origin is at
/// `position`; lines go down.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Text {
    pub glyphs: Vec<TextGlyph>,
    pub position: Vec3,
    /// Em size in world units (CSS px in the overlay).
    pub size: f32,
    /// Counter-clockwise rotation around `position`, in radians (clockwise
    /// on screen in the overlay).
    pub rotation: f32,
    /// Linear RGBA.
    pub color: [f32; 4],
    /// Sorted together with sprites: higher layers draw over lower ones.
    pub layer: i32,
    /// In [`Overlay::texts`]: the index of the clip rectangle in
    /// [`Overlay::clips`] the text is cut to, per pixel (`None`: the whole
    /// output). Ignored for world text.
    pub clip: Option<u32>,
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
    /// What its contents were made from, when one writer made them all and
    /// can name it (og-runtime: the UI frame's generation and its
    /// anchored shifts); anything else that writes into it sets `None`.
    /// The renderer keeps what it made from an overlay with the same key
    /// at the same target instead of making and uploading it again; its
    /// `ui.time`, `ui.tick` and `ui.tick_fraction` are read each frame.
    pub key: Option<u64>,
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
            key: None,
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
