//! Rive artboards (`docs/ui.md` §3.3), converted from a `.riv` at cook
//! time (og-cook's `riv`), never by the Rive runtime: an [`Artboard`] is
//! its objects as [`Node`]s (transforms, paths and their vertices, paints
//! and their colours, nested artboards) and its animations as keyframe
//! [`Track`]s on the nodes' [`Values`]. A [`Player`] advances one on the
//! tick and draws it as UI shapes ([`og_draw::UiGeometry::Path`]): it is
//! deterministic (a fixed step, no clock) and its objects are named
//! elements for dump_ui, describe and pick ([`Player::elements`]).
//!
//! The path construction, the vertex control points and the cubic easing
//! follow rive-runtime's `Path::buildPath`, its vertices and
//! `CubicInterpolatorSolver` (MIT, `third_party/rive/`), so a converted
//! artboard draws and moves as the Rive editor shows it.

mod codec;
mod player;
mod text;

pub use codec::{check, decode, encode};
pub use player::{Drawn, Player, RiveElement, TextDraw};

use og_core::glam::Vec2;
use og_draw::{UiCap, UiFillRule};

/// An artboard: its frame, its objects and its animations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Artboard {
    pub name: String,
    /// Its width and height, CSS px (one Rive unit is one CSS px).
    pub size: Vec2,
    /// Where the content's (0, 0) sits in the frame, fractions of `size`.
    pub origin: Vec2,
    /// Whether the content is cut to the frame.
    pub clip: bool,
    /// How the frame is placed in the box it is drawn in (the import
    /// settings' `fit` and `align`; `align` fractions of the free space).
    pub fit: Fit,
    pub align: Vec2,
    /// Its accessibility role and name (the import settings').
    pub role: String,
    pub label: String,
    /// Its objects, every parent before its children; node 0 is the
    /// artboard itself ([`Kind::Artboard`]).
    pub nodes: Vec<Node>,
    /// The drawn nodes (shapes and nested artboards), bottom to top.
    pub draws: Vec<u32>,
    pub animations: Vec<Animation>,
    /// Its state machines by name, each with the animation it plays (phase
    /// 1 converts a machine whose entry plays one animation).
    pub machines: Vec<(String, Option<u32>)>,
    /// The animation it plays by default (the import settings' machine,
    /// else the file's default, else its first animation).
    pub play: Option<u32>,
    /// The artboards its [`Kind::Nested`] nodes draw.
    pub nested: Vec<Artboard>,
    /// The images its [`Kind::Image`] nodes draw.
    pub images: Vec<Image>,
    /// The fonts its [`Kind::TextStyle`] nodes set text in.
    pub fonts: Vec<Font>,
    /// The words its text runs show, at rest and as keyed
    /// ([`Values::text`] is an index here).
    pub strings: Vec<String>,
}

/// A font an artboard's text is set in: a font the cook made from the
/// `.riv` (the file's own, or the project font its settings name for a
/// referenced one), with the face's line metrics as Rive lays lines out.
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    /// The font's asset id, `<file>.riv#font<n>`.
    pub id: String,
    /// Baseline to the face's ascender and descender, em (both positive):
    /// its horizontal header's, as Rive reads them.
    pub ascent: f32,
    pub descent: f32,
    /// The font once loaded ([`Artboard::bind_fonts`]); not in the blob.
    /// Text in an unbound font is laid out and drawn as if it had none.
    pub font: Option<og_text::text::UiFont>,
}

/// An image an artboard draws: a texture cooked from the `.riv`.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    /// The texture's asset id, `<file>.riv#image<n>`.
    pub id: String,
    /// Its size in pixels: one pixel per Rive unit, as Rive draws it.
    pub size: Vec2,
    /// Its texture once loaded ([`Artboard::bind_images`]); not in the
    /// blob. An unbound image draws nothing.
    pub texture: Option<og_draw::TextureHandle>,
}

impl Artboard {
    /// Gives every image of this artboard and the ones it nests its
    /// texture, by id (the loader's: `None` leaves it undrawn).
    pub fn bind_images(&mut self, texture: &mut dyn FnMut(&str) -> Option<og_draw::TextureHandle>) {
        for i in &mut self.images {
            i.texture = texture(&i.id);
        }
        for n in &mut self.nested {
            n.bind_images(texture);
        }
    }

    /// Gives every font of this artboard and the ones it nests its loaded
    /// font, by id (the loader's: `None` leaves its text undrawn).
    pub fn bind_fonts(&mut self, font: &mut dyn FnMut(&str) -> Option<og_text::text::UiFont>) {
        for f in &mut self.fonts {
            f.font = font(&f.id);
        }
        for n in &mut self.nested {
            n.bind_fonts(font);
        }
    }
}

/// How a text's box is sized (Rive's `TextSizing`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sizing {
    /// As wide and tall as its lines; nothing wraps.
    #[default]
    AutoWidth,
    /// [`Values::width`] wide, as tall as its lines.
    AutoHeight,
    /// [`Values::width`] by [`Values::height`].
    Fixed,
}

/// What a text does with lines past its box (Rive's `TextOverflow`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Drawn past it.
    #[default]
    Visible,
    /// Lines that do not fit whole are not drawn.
    Hidden,
    /// Lines that do not fit at all are not drawn; the rest is cut to it.
    Clipped,
    /// The last line that fits ends in "...".
    Ellipsis,
    /// Scaled down to fit.
    Fit,
    /// Set at the largest whole font size that fits.
    FitFontSize,
}

/// Where a text's lines sit across its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HAlign {
    #[default]
    Left,
    Right,
    Center,
}

/// Where a text's lines sit down its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VAlign {
    #[default]
    Top,
    Bottom,
    Middle,
}

/// The rest of a [`Kind::Text`]'s layout, which no animation keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextLayout {
    pub sizing: Sizing,
    pub overflow: Overflow,
    pub align: HAlign,
    pub valign: VAlign,
    /// Whether lines wrap at the box's width ([`Sizing::AutoWidth`]
    /// never does).
    pub wrap: bool,
    /// Whether a word wider than a line is cut where it overflows when it
    /// is alone on its line (Rive's `breakWord`), or overflows whole
    /// (`normal`).
    pub break_words: bool,
    /// Whether the text's (0, 0) is its first baseline (else its box's top).
    pub baseline: bool,
    /// [`Overflow::Fit`] scales about the first baseline (else the top).
    pub fit_from_baseline: bool,
}

/// How an artboard's frame is scaled into its box.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fit {
    /// Stretched to the box.
    Fill,
    /// As large as fits inside.
    #[default]
    Contain,
    /// As small as covers it.
    Cover,
    FitWidth,
    FitHeight,
    /// One Rive unit per CSS px.
    None,
    /// Contain, never above its own size.
    ScaleDown,
}

impl Fit {
    /// Every fit, in codec order, with its settings word.
    pub const ALL: [(Fit, &'static str); 7] = [
        (Fit::Fill, "fill"),
        (Fit::Contain, "contain"),
        (Fit::Cover, "cover"),
        (Fit::FitWidth, "fit-width"),
        (Fit::FitHeight, "fit-height"),
        (Fit::None, "none"),
        (Fit::ScaleDown, "scale-down"),
    ];
}

/// One object of an artboard.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Its name in the Rive file (possibly empty); its element id is
    /// `<artboard>/<name>`.
    pub name: String,
    /// Its parent's index (node 0's is 0).
    pub parent: u32,
    pub kind: Kind,
    /// Its values at rest; animations write over them.
    pub values: Values,
}

/// What a node is.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// The artboard, with its background paints.
    Artboard { paints: Vec<u32> },
    /// A transform for its children.
    Group,
    /// Its paths filled and stroked by its paints, in order.
    Shape {
        paths: Vec<u32>,
        paints: Vec<u32>,
        hidden: bool,
        blend: og_draw::UiBlend,
    },
    /// An outline in the shape it belongs to.
    Path { outline: Outline, closed: bool },
    /// A vertex of an [`Outline::Points`] path.
    Vertex(Vertex),
    /// A paint of its shape: the area, by its rule.
    Fill {
        rule: UiFillRule,
        shader: u32,
        visible: bool,
    },
    /// A paint of its shape: the outline, [`Values::thickness`] wide.
    Stroke {
        cap: UiCap,
        join: Join,
        /// Whether the shape's transform scales its width.
        scaled: bool,
        shader: u32,
        visible: bool,
    },
    /// A paint's colour, [`Values::color`].
    Solid,
    /// A paint's gradient from [`Values::start`] to [`Values::end`] (a
    /// radial one: centred on `start`, of radius |end − start|), in the
    /// shape's space.
    Gradient { radial: bool, stops: Vec<u32> },
    /// A gradient's stop: [`Values::color`] at [`Values::position`].
    Stop,
    /// An image ([`Artboard::images`]) at its size, placed by
    /// [`Values::origin`] (fractions of the image the node sits on).
    Image { image: u32 },
    /// Another artboard ([`Artboard::nested`]) drawn at this transform, its
    /// animations playing.
    Nested {
        artboard: u32,
        animations: Vec<NestedAnimation>,
    },
    /// A text: its runs ([`Kind::TextRun`]) in order, laid out in a box
    /// [`Values::width`] by [`Values::height`], placed about
    /// [`Values::origin`] (fractions of the box), its paragraphs
    /// [`Values::spacing`] apart.
    Text { runs: Vec<u32>, layout: TextLayout },
    /// A run of a text: [`Values::text`] in its style ([`Kind::TextStyle`]).
    TextRun { style: u32 },
    /// A text's style: [`Values::font`] at [`Values::font_size`],
    /// [`Values::line_height`] (negative: the font's) and
    /// [`Values::letter_spacing`], its glyphs drawn by its paints.
    TextStyle { paints: Vec<u32> },
}

/// The outline of a [`Kind::Path`], in the path's space.
#[derive(Clone, Debug, PartialEq)]
pub enum Outline {
    /// Through its vertices.
    Points {
        vertices: Vec<u32>,
    },
    /// [`Values::width`] by [`Values::height`] about [`Values::origin`],
    /// each corner rounded by its [`Values::corners`] (top left, top right,
    /// bottom right, bottom left).
    Rect,
    Ellipse,
    /// Its apex at the top middle.
    Triangle,
    /// `points` corners on the ellipse of the box, the first at the top,
    /// rounded by `corners[0]`.
    Polygon {
        points: u32,
    },
    /// `points` outer corners and as many inner ones at [`Values::inner`] of
    /// the box.
    Star {
        points: u32,
    },
}

/// A path vertex's kind: its control points come from [`Values::x`],
/// [`Values::y`] and its rotations and distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vertex {
    /// A corner, rounded by [`Values::radius`] (negative: cut inwards).
    Straight,
    /// In: `in_rotation`, `in_distance`; out: `rotation`, `out_distance`.
    Detached,
    /// One `rotation`, the same `in_distance` either side.
    Mirrored,
    /// One `rotation`, `in_distance` back and `out_distance` forward.
    Asymmetric,
}

/// A stroke's corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Join {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// An animation a nested artboard plays: one of its own, at `speed`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NestedAnimation {
    pub animation: u32,
    pub speed: f32,
    pub playing: bool,
}

/// What a node has that animations change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Values {
    pub x: f32,
    pub y: f32,
    /// Radians, clockwise on screen.
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    /// Multiplies its children's.
    pub opacity: f32,
    pub width: f32,
    pub height: f32,
    pub origin: Vec2,
    pub corners: [f32; 4],
    pub inner: f32,
    /// A straight vertex's corner radius.
    pub radius: f32,
    pub in_rotation: f32,
    pub in_distance: f32,
    pub out_distance: f32,
    pub thickness: f32,
    /// 0xAARRGGBB, sRGB, straight alpha (Rive's colour).
    pub color: u32,
    pub position: f32,
    pub start: Vec2,
    pub end: Vec2,
    /// A text style's size (CSS px per em), line height (CSS px; negative:
    /// the font's ascent and descent) and the space after each glyph.
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    /// A text's space between paragraphs.
    pub spacing: f32,
    /// A run's words: an index into [`Artboard::strings`].
    pub text: u32,
    /// A style's font: an index into [`Artboard::fonts`] ([`NO_FONT`]: none).
    pub font: u32,
}

/// [`Values::font`] of a style with no font: its runs are not drawn.
pub const NO_FONT: u32 = u32::MAX;

impl Default for Values {
    fn default() -> Values {
        Values {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
            width: 0.0,
            height: 0.0,
            origin: Vec2::splat(0.5),
            corners: [0.0; 4],
            inner: 0.5,
            radius: 0.0,
            in_rotation: 0.0,
            in_distance: 0.0,
            out_distance: 0.0,
            thickness: 1.0,
            color: 0xFF74_7474,
            position: 0.0,
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            font_size: 12.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            spacing: 0.0,
            text: 0,
            font: NO_FONT,
        }
    }
}

/// A value an animation keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prop {
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
    Width,
    Height,
    OriginX,
    OriginY,
    /// A rectangle's corner (0 top left … 3 bottom left); a polygon's 0.
    Corner(u8),
    Inner,
    Radius,
    InRotation,
    InDistance,
    OutDistance,
    Thickness,
    Color,
    Position,
    StartX,
    StartY,
    EndX,
    EndY,
    FontSize,
    LineHeight,
    LetterSpacing,
    Spacing,
    /// A run's words, keyed as indices into [`Artboard::strings`] (held).
    Text,
    /// A style's font, keyed as indices into [`Artboard::fonts`] (held).
    Font,
}

impl Prop {
    /// Every prop, in codec order.
    pub const ALL: [Prop; 32] = [
        Prop::X,
        Prop::Y,
        Prop::Rotation,
        Prop::ScaleX,
        Prop::ScaleY,
        Prop::Opacity,
        Prop::Width,
        Prop::Height,
        Prop::OriginX,
        Prop::OriginY,
        Prop::Corner(0),
        Prop::Corner(1),
        Prop::Corner(2),
        Prop::Corner(3),
        Prop::Inner,
        Prop::Radius,
        Prop::InRotation,
        Prop::InDistance,
        Prop::OutDistance,
        Prop::Thickness,
        Prop::Color,
        Prop::Position,
        Prop::StartX,
        Prop::StartY,
        Prop::EndX,
        Prop::EndY,
        Prop::FontSize,
        Prop::LineHeight,
        Prop::LetterSpacing,
        Prop::Spacing,
        Prop::Text,
        Prop::Font,
    ];
}

impl Values {
    /// Sets a number prop.
    pub fn set(&mut self, p: Prop, v: f32) {
        match p {
            Prop::X => self.x = v,
            Prop::Y => self.y = v,
            Prop::Rotation => self.rotation = v,
            Prop::ScaleX => self.scale_x = v,
            Prop::ScaleY => self.scale_y = v,
            Prop::Opacity => self.opacity = v,
            Prop::Width => self.width = v,
            Prop::Height => self.height = v,
            Prop::OriginX => self.origin.x = v,
            Prop::OriginY => self.origin.y = v,
            Prop::Corner(i) => self.corners[usize::from(i & 3)] = v,
            Prop::Inner => self.inner = v,
            Prop::Radius => self.radius = v,
            Prop::InRotation => self.in_rotation = v,
            Prop::InDistance => self.in_distance = v,
            Prop::OutDistance => self.out_distance = v,
            Prop::Thickness => self.thickness = v,
            // Colours are keyed as colours ([`Key::color`]).
            Prop::Color => {}
            Prop::Position => self.position = v,
            Prop::StartX => self.start.x = v,
            Prop::StartY => self.start.y = v,
            Prop::EndX => self.end.x = v,
            Prop::EndY => self.end.y = v,
            Prop::FontSize => self.font_size = v,
            Prop::LineHeight => self.line_height = v,
            Prop::LetterSpacing => self.letter_spacing = v,
            Prop::Spacing => self.spacing = v,
            // Indices, exact in an f32 below 2^24; NO_FONT saturates back.
            Prop::Text => self.text = v as u32,
            Prop::Font => self.font = v as u32,
        }
    }
}

impl Values {
    /// A number prop's value ([`Prop::Color`]: 0; colours are keyed as colours).
    pub fn get(&self, p: Prop) -> f32 {
        match p {
            Prop::X => self.x,
            Prop::Y => self.y,
            Prop::Rotation => self.rotation,
            Prop::ScaleX => self.scale_x,
            Prop::ScaleY => self.scale_y,
            Prop::Opacity => self.opacity,
            Prop::Width => self.width,
            Prop::Height => self.height,
            Prop::OriginX => self.origin.x,
            Prop::OriginY => self.origin.y,
            Prop::Corner(i) => self.corners[usize::from(i & 3)],
            Prop::Inner => self.inner,
            Prop::Radius => self.radius,
            Prop::InRotation => self.in_rotation,
            Prop::InDistance => self.in_distance,
            Prop::OutDistance => self.out_distance,
            Prop::Thickness => self.thickness,
            Prop::Color => 0.0,
            Prop::Position => self.position,
            Prop::StartX => self.start.x,
            Prop::StartY => self.start.y,
            Prop::EndX => self.end.x,
            Prop::EndY => self.end.y,
            Prop::FontSize => self.font_size,
            Prop::LineHeight => self.line_height,
            Prop::LetterSpacing => self.letter_spacing,
            Prop::Spacing => self.spacing,
            Prop::Text => self.text as f32,
            Prop::Font => self.font as f32,
        }
    }
}

/// How an animation runs past its end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Loop {
    /// Stops on its last frame.
    #[default]
    OneShot,
    Loop,
    /// Back and forth.
    PingPong,
}

/// A timeline of keyed values.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub name: String,
    pub fps: u32,
    /// Its length in frames.
    pub duration: u32,
    pub speed: f32,
    pub looping: Loop,
    /// The frames it plays (`start..end`), when not all of them.
    pub work: Option<(u32, u32)>,
    pub tracks: Vec<Track>,
}

impl Animation {
    /// Its first and last second.
    pub fn span(&self) -> (f32, f32) {
        let (a, b) = self.work.unwrap_or((0, self.duration));
        let fps = self.fps.max(1) as f32;
        (a as f32 / fps, b as f32 / fps)
    }
}

/// One node's prop over time.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub node: u32,
    pub prop: Prop,
    /// In time order.
    pub keys: Vec<Key>,
}

/// A keyframe: the value at `seconds` and how it eases to the next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    pub seconds: f32,
    /// The number, or for [`Prop::Color`] 0xAARRGGBB in [`Key::color`].
    pub value: f32,
    pub color: u32,
    pub ease: Ease,
}

/// How a key moves to the next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    /// Keeps its value until the next key.
    Hold,
    Linear,
    /// A cubic Bézier of time, (0, 0) to (1, 1) through (x1, y1), (x2, y2).
    Cubic {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    /// A cubic Bézier whose y runs through the values themselves, from this
    /// key's to the next's through `y1`, `y2` (Rive's value interpolator).
    CubicValue {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}
