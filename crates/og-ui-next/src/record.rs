//! An element's computed look (its record) and the parts it is made of:
//! what its classes set, folded in order, and what its text inherits.

use rune_taffy::CompactStyle;
use taffy::LengthPercentageAuto;

/// A linear RGBA colour.
pub type Rgba = [f32; 4];

/// The margins `space-x-*` and `space-y-*` give each of an element's
/// children but the last, under a child's own margin on that side: the
/// length and whether it is the left (x) or the top (y) one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spacing {
    pub x: Option<(LengthPercentageAuto, bool)>,
    pub y: Option<(LengthPercentageAuto, bool)>,
}

/// An element's text properties. Text inherits: a field its own rules do
/// not set (a bit clear in `set`) is its parent's, applied when the
/// element's record is applied, never matched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Text {
    pub weight: u16,
    pub tracking_px: f32,
    pub tracking_em: f32,
    /// A multiple of the font size, or with `line_px` CSS px.
    pub line: f32,
    pub line_px: bool,
    pub nowrap: bool,
    /// `text-align`: 0 start, 1 center, 2 right, 3 left, 4 justify.
    pub align: u8,
    /// `text-overflow`: 1 an ellipsis where a line is cut, 0 none.
    pub overflow: u8,
    /// `-og-scroll`'s cycle and pause, ms.
    pub scroll: [f32; 2],
    /// 0 none, 1 uppercase, 2 lowercase, 3 capitalize.
    pub transform: u8,
    /// Where a line may break inside a word (`hyphens-*`): CSS's
    /// `manual` (soft hyphens only) unless set. The line breaker reads it.
    pub hyphens: nsh_tailwind::Hyphens,
    /// The most lines it shows (`line-clamp-*`), the last ending in an
    /// ellipsis when more follow; 0 all. Its text children take it.
    pub clamp: u16,
    /// Its `font-family` list (CSS, interned by NSH); `None`: the theme's
    /// sans.
    pub family: Option<nsh_tailwind::style::FontFamily>,
    pub italic: bool,
    /// `font-variant-*`: NSH's `FontVariant` bits.
    pub variant: u8,
    /// `overflow-wrap`: 0 normal, 1 break-word, 2 anywhere.
    pub wrap: u8,
    /// Which fields its own rules set: bit k for [`Text::WEIGHT`] and on.
    pub set: u16,
    /// Its font size was set by its own rules.
    pub sized: bool,
}

impl Text {
    pub const WEIGHT: u16 = 1;
    pub const TRACKING: u16 = 2;
    pub const LINE: u16 = 4;
    pub const NOWRAP: u16 = 8;
    pub const ALIGN: u16 = 16;
    pub const OVERFLOW: u16 = 32;
    pub const TRANSFORM: u16 = 64;
    pub const HYPHENS: u16 = 128;
    pub const CLAMP: u16 = 256;
    pub const FAMILY: u16 = 512;
    pub const ITALIC: u16 = 1024;
    pub const VARIANT: u16 = 2048;
    pub const WRAP: u16 = 4096;

    /// Its letter spacing at `size`, CSS px.
    pub fn tracking(&self, size: f32) -> f32 {
        self.tracking_px + self.tracking_em * size
    }

    /// Its line height at `size`, CSS px; negative for CSS `normal`, the
    /// height the line's own fonts give it.
    pub fn line_height(&self, size: f32) -> f32 {
        if self.line_px {
            self.line
        } else {
            self.line * size
        }
    }

    /// This text over `parent`'s: the fields it does not set are the
    /// parent's.
    pub fn over(&self, parent: &Text) -> Text {
        let mut t = *parent;
        let own = |bit| self.set & bit != 0;
        if own(Text::WEIGHT) {
            t.weight = self.weight;
        }
        if own(Text::TRACKING) {
            (t.tracking_px, t.tracking_em) = (self.tracking_px, self.tracking_em);
        }
        if own(Text::LINE) {
            (t.line, t.line_px) = (self.line, self.line_px);
        }
        if own(Text::NOWRAP) {
            t.nowrap = self.nowrap;
        }
        if own(Text::ALIGN) {
            t.align = self.align;
        }
        if own(Text::OVERFLOW) {
            (t.overflow, t.scroll) = (self.overflow, self.scroll);
        }
        if own(Text::TRANSFORM) {
            t.transform = self.transform;
        }
        if own(Text::HYPHENS) {
            t.hyphens = self.hyphens;
        }
        if own(Text::CLAMP) {
            t.clamp = self.clamp;
        }
        if own(Text::FAMILY) {
            t.family = self.family;
        }
        if own(Text::ITALIC) {
            t.italic = self.italic;
        }
        if own(Text::VARIANT) {
            t.variant = self.variant;
        }
        if own(Text::WRAP) {
            t.wrap = self.wrap;
        }
        t.set = parent.set | self.set;
        t.sized = self.sized;
        t
    }
}

impl Default for Text {
    fn default() -> Self {
        Text {
            weight: 400,
            tracking_px: 0.0,
            tracking_em: 0.0,
            // CSS's initial `normal`.
            line: -1.0,
            line_px: false,
            nowrap: false,
            align: 0,
            overflow: 0,
            scroll: [0.0; 2],
            transform: 0,
            hyphens: nsh_tailwind::Hyphens::Manual,
            clamp: 0,
            family: None,
            italic: false,
            variant: 0,
            wrap: 0,
            set: 0,
            sized: false,
        }
    }
}

/// The most stops a background gradient keeps: a two-colour gradient mixed
/// in Oklab (NSH's subdivided stops) fits whole; more are sampled evenly,
/// the first and last kept.
pub const GRADIENT_STOPS: usize = 16;

/// A gradient's geometry (CSS Images 3 and 4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Along a line at its angle (`bg-linear-*`, `bg-gradient-to-*`).
    Linear,
    /// Out from its centre to an ending shape sized by `size`, a circle or
    /// an ellipse with the box's proportions (`bg-radial-*`).
    Radial {
        circle: bool,
        size: nsh_tailwind::effects::GradientSize,
    },
    /// Around its centre, starting at its angle, stops placed in turns
    /// (`bg-conic-*`).
    Conic,
}

/// A gradient background, over the fill colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    pub shape: Shape,
    /// Radians, CSS's: 0 up, clockwise: a linear gradient's direction, a
    /// conic one's start.
    pub angle: f32,
    /// `to <corner>`: x and y, -1 or 1, aimed at the box's corner at paint
    /// time; `[0, 0]`: `angle`.
    pub corner: [i8; 2],
    /// A radial or conic gradient's centre, fractions of the box from its
    /// top left.
    pub center: [f32; 2],
    pub count: u8,
    /// (offset 0..1, linear RGBA), `count` of them.
    pub stops: [(f32, Rgba); GRADIENT_STOPS],
}

impl Gradient {
    pub fn stops(&self) -> &[(f32, Rgba)] {
        &self.stops[..usize::from(self.count)]
    }
}

/// The most box-shadow layers a record keeps.
pub const SHADOWS: usize = 4;

/// A shadow layer (`box-shadow`, `ring-*`, `text-shadow`); a colour of 0
/// alpha draws nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shadow {
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Rgba,
    pub inset: bool,
}

/// An outline: outside the border, `offset` out, taking no space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Outline {
    pub width: f32,
    pub offset: f32,
    pub color: Rgba,
}

/// A paint-time transform around the box (layout never moves).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translate: [f32; 2],
    /// A translation by fractions of the box's own size (CSS's
    /// percentages, `-translate-x-1/2`), on top of `translate`.
    pub percent: [f32; 2],
    pub scale: [f32; 2],
    /// Radians, clockwise.
    pub rotate: f32,
    /// Fractions of the box.
    pub origin: [f32; 2],
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translate: [0.0; 2],
            percent: [0.0; 2],
            scale: [1.0; 2],
            rotate: 0.0,
            origin: [0.5; 2],
        }
    }
}

impl Transform {
    pub fn is_identity(&self) -> bool {
        self.translate == [0.0; 2]
            && self.percent == [0.0; 2]
            && self.scale == [1.0; 2]
            && self.rotate == 0.0
    }
}

/// A text's outline (`-webkit-text-stroke`): drawn under its fill, so
/// `width / 2` shows outside each glyph (`paint-order: stroke fill`, the
/// only order). 0 width or alpha: none.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextStroke {
    /// CSS px, the stroke's full width.
    pub width: f32,
    pub color: Rgba,
}

/// A text's decoration lines.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Decoration {
    /// Bits: [`Decoration::UNDERLINE`], `OVERLINE`, `LINE_THROUGH`.
    pub line: u8,
    /// 0 alpha: the text's ink.
    pub color: Rgba,
    /// 0: from the font size.
    pub thickness: f32,
}

impl Decoration {
    pub const UNDERLINE: u8 = 1;
    pub const OVERLINE: u8 = 2;
    pub const LINE_THROUGH: u8 = 4;
}

/// What colours a box's image.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tint {
    /// Its own colours.
    #[default]
    Own,
    /// Times the text colour (an icon mask).
    Ink,
    /// Times the box's fill (`bg-*`), which then is not drawn under it: a
    /// pack's plate art follows the theme and scheme.
    Fill,
}

/// A box's image (generated panel, button, frame and slot art): a texture
/// region, nine-sliced, over the fill and under the border. Its texture is
/// a handle resolved at run time, so the TSX applier writes it; no sheet
/// does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slice {
    /// The texture's slot (`og_draw::TextureHandle::slot`); 0 none.
    pub texture: u32,
    /// Its region, u0 v0 u1 v1.
    pub uv: [f32; 4],
    /// Slice insets in texture pixels, top, right, bottom, left; 0 none.
    pub slice: [f32; 4],
    /// How wide each slice draws, CSS px; all 0: the slices' own sizes.
    pub width: [f32; 4],
    /// Draw the centre slice.
    pub fill: bool,
    /// Tile edges and centre in whole tiles instead of stretching.
    pub tile: bool,
    /// What colours it: its own, or a white mask tinted.
    pub tint: Tint,
    /// A radial wipe: the fraction shown, 0..1 (1 whole), from `wipe_start`
    /// (radians, 0 up, clockwise), counter-clockwise with `wipe_ccw`.
    pub wipe: f32,
    pub wipe_start: f32,
    pub wipe_ccw: bool,
    /// Its texture's size in texels, as its source resolved; 0 unknown.
    pub size: [f32; 2],
}

impl Default for Slice {
    fn default() -> Self {
        Slice {
            texture: 0,
            uv: [0.0, 0.0, 1.0, 1.0],
            slice: [0.0; 4],
            width: [0.0; 4],
            fill: true,
            tile: false,
            tint: Tint::Own,
            wipe: 1.0,
            wipe_start: 0.0,
            wipe_ccw: false,
            size: [0.0; 2],
        }
    }
}

/// An element's paint: every field the paint pass reads, `Copy`, so motion
/// holds an animated copy of it per node and repaint reads either.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skin {
    pub fill: Rgba,
    /// A gradient over `fill` (`background: linear-gradient(…), <colour>`).
    pub gradient: Option<Gradient>,
    pub ink: Rgba,
    /// Corner radii: top left, top right, bottom right, bottom left.
    pub radius: [f32; 4],
    /// Border widths, top, right, bottom, left (also in the layout).
    pub border: [f32; 4],
    /// The border's colour, and `border_side`'s on the sides in
    /// `border_sides` (bits: 1 top, 2 right, 4 bottom, 8 left), as
    /// `border-t-*` over `border-*` writes them; motion animates the first.
    pub border_color: Rgba,
    pub border_side: Rgba,
    pub border_sides: u8,
    /// Box shadows, `shadow_count` of them, first on top.
    pub shadows: [Shadow; SHADOWS],
    pub shadow_count: u8,
    pub outline: Outline,
    pub transform: Transform,
    /// `backdrop-filter: blur()`, CSS px; 0 none.
    pub backdrop_blur: f32,
    pub text_shadow: Shadow,
    pub decoration: Decoration,
    pub text_stroke: TextStroke,
    pub opacity: f32,
    /// An SVG shape's fill and stroke paint, and its stroke's width.
    pub paint: Rgba,
    pub stroke: Rgba,
    pub stroke_width: f32,
    pub image: Slice,
}

impl Default for Skin {
    fn default() -> Self {
        Skin {
            fill: [0.0; 4],
            gradient: None,
            ink: [0.0, 0.0, 0.0, 1.0],
            radius: [0.0; 4],
            border: [0.0; 4],
            border_color: [0.0; 4],
            border_side: [0.0; 4],
            border_sides: 0,
            shadows: [Shadow::default(); SHADOWS],
            shadow_count: 0,
            outline: Outline::default(),
            transform: Transform::default(),
            backdrop_blur: 0.0,
            text_shadow: Shadow::default(),
            decoration: Decoration::default(),
            text_stroke: TextStroke::default(),
            opacity: 1.0,
            paint: [0.0, 0.0, 0.0, 1.0],
            stroke: [0.0; 4],
            stroke_width: 1.0,
            image: Slice::default(),
        }
    }
}

/// An element's computed look: what its rules set, folded in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub layout: CompactStyle,
    pub skin: Skin,
    pub font_size: f32,
    /// The sides of its `inset` (0) and `padding` (1) that add the HUD's
    /// safe-area inset (`env(-og-hud-inset-left)`), bits top, right,
    /// bottom, left: the one runtime input a record leaves open.
    pub safe: [u8; 2],
    pub text: Text,
    /// Its ink was set by its own rules; else it inherits its parent's.
    pub inked: bool,
    /// Its text shadow and its text outline were set by its own rules;
    /// else each inherits its parent's, as CSS's do.
    pub shadowed: bool,
    pub stroked: bool,
    /// `position: fixed`: placed against the viewport.
    pub fixed: bool,
    /// Its `transition-*`, `duration-*`, `ease-*`, `delay-*` and
    /// `animate-*`.
    pub motion: crate::motion::Spec,
    /// A `<Line>`'s stroke (`stroke-*`): its colour, `None` for the ink
    /// (`currentColor`, the default), and its width in CSS px.
    pub stroke: Option<Rgba>,
    pub stroke_width: f32,
    /// The pointer over it (`cursor-*`); `Auto` takes its parent's.
    pub cursor: nsh_tailwind::effects::Cursor,
    /// Its `z-*`: `None` for `z-auto` (the default), which stacks nothing;
    /// `Some(0)` stacks at 0. The tree's stacking contexts read it.
    pub z_index: Option<i32>,
    /// Its `mask-[<gradient>]` (CSS's `mask-image`): the gradient's alpha
    /// over its border box scales everything it and its subtree draw.
    pub mask: Option<Gradient>,
    /// Its `scheme-dark` (true) or `scheme-light`: the scheme its subtree
    /// is drawn in whatever the page's, for art that is one picture in
    /// both; `None` follows its parent.
    pub scheme: Option<bool>,
    /// The scheme it was resolved in.
    pub dark: bool,
    /// `invisible` (CSS's `visibility: hidden`): it and its subtree are
    /// laid out but neither drawn nor touched.
    pub hidden: bool,
    /// `pointer-events-none` (false) or `pointer-events-auto` (true): whether
    /// a press, a hover or the cursor reaches it and the subtree that sets
    /// none; `None` follows its parent, as CSS's inherits.
    pub touch: Option<bool>,
    /// Its `fill-*`, the paint of the icons in it: `Some(None)` the ink
    /// (`fill-current`), `Some(Some(transparent))` none (`fill-none`);
    /// `None` follows its parent, as CSS's `fill` inherits.
    pub fill: Option<Option<Rgba>>,
    /// Its `space-x-*` and `space-y-*`, for its children.
    pub space: Spacing,
    /// Its `object-*` (CSS's `object-fit` and `object-position`): how its
    /// image's source fits its box; `None` stretches it (`fill`).
    pub object: Option<(
        nsh_tailwind::ResizeMode,
        nsh_tailwind::style::ObjectPosition,
    )>,
    /// Its colour filters (`brightness-*`, `contrast-*`, `grayscale`,
    /// `hue-rotate-*`, `invert`, `saturate-*`, `sepia`, `opacity-*` in
    /// `filter`), composed in order into one matrix over everything it
    /// and its subtree draw; `None` none.
    pub filter: Option<og_draw::paint::ColorMatrix>,
}

impl Record {
    /// Its layout with the HUD's safe-area `insets` (top, right, bottom,
    /// left) added where its rules read them.
    pub fn layout_in(&self, insets: [f32; 4]) -> CompactStyle {
        if self.safe == [0; 2] {
            return self.layout.clone();
        }
        let mut s = taffy::Style::from(&self.layout);
        let add = |mask: u8, k: usize| if mask & (1 << k) != 0 { insets[k] } else { 0.0 };
        let [t, r, b, l] = [0, 1, 2, 3].map(|k| add(self.safe[0], k));
        let grow = |v: &mut LengthPercentageAuto, by: f32| {
            let raw = v.into_raw();
            if by != 0.0 && raw.has_calc() {
                *v = LengthPercentageAuto::calc(rune_taffy::calc::plus(raw.calc_value(), by));
            } else if by != 0.0 && !v.is_auto() {
                *v = LengthPercentageAuto::length(raw.value() + by);
            }
        };
        grow(&mut s.inset.top, t);
        grow(&mut s.inset.right, r);
        grow(&mut s.inset.bottom, b);
        grow(&mut s.inset.left, l);
        let [t, r, b, l] = [0, 1, 2, 3].map(|k| add(self.safe[1], k));
        let pad = |v: &mut taffy::LengthPercentage, by: f32| {
            let raw = v.into_raw();
            if by != 0.0 && raw.has_calc() {
                *v = taffy::LengthPercentage::calc(rune_taffy::calc::plus(raw.calc_value(), by));
            } else if by != 0.0 {
                *v = taffy::LengthPercentage::length(raw.value() + by);
            }
        };
        pad(&mut s.padding.top, t);
        pad(&mut s.padding.right, r);
        pad(&mut s.padding.bottom, b);
        pad(&mut s.padding.left, l);
        CompactStyle::from(&s)
    }
}

impl Default for Record {
    fn default() -> Self {
        // Every box is its anchored descendants' containing block, as in
        // og-ui: `position` other than absolute and fixed is relative.
        let mut base = taffy::Style::from(&CompactStyle::default());
        base.position = taffy::Position::Relative;
        Record {
            layout: CompactStyle::from(&base),
            skin: Skin::default(),
            font_size: 16.0,
            safe: [0; 2],
            text: Text::default(),
            inked: false,
            shadowed: false,
            stroked: false,
            fixed: false,
            motion: crate::motion::Spec::default(),
            stroke: None,
            stroke_width: 1.0,
            cursor: nsh_tailwind::effects::Cursor::Auto,
            z_index: None,
            mask: None,
            hidden: false,
            touch: None,
            fill: None,
            space: Spacing::default(),
            object: None,
            filter: None,
            scheme: None,
            dark: false,
        }
    }
}
