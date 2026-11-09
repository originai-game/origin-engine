use super::*;

/// Extra drawing of a widget, in its own frame, after its background.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shape {
    /// A decorative rectangle with an exact legacy fallback until dressed.
    DressedRect {
        frame: Frame,
        part: crate::dress::PartRef,
        radius: f32,
        color: Srgba,
    },
    /// A rounded rectangle (`radius` ≥ half the short side: a capsule or disc).
    Rect {
        frame: Frame,
        radius: f32,
        color: Srgba,
    },
    /// A solid bar rotated by `angle` (radians, clockwise on screen) about its centre.
    Bar {
        part: crate::dress::PartRef,
        frame: Frame,
        angle: f32,
        color: Srgba,
    },
    /// The half of a disc on the side `angle` points to (0: the right half,
    /// clockwise on screen).
    Half {
        part: crate::dress::PartRef,
        frame: Frame,
        angle: f32,
        color: Srgba,
    },
    /// A ring's arc on the circle the frame holds: `sweep` radians from
    /// `start` (clockwise from +x on screen), `thickness` CSS px inwards,
    /// round ends. Dressed by `part` as a decorative subpart (its colour,
    /// opacity and box).
    Arc {
        part: crate::dress::PartRef,
        frame: Frame,
        start: f32,
        sweep: f32,
        thickness: f32,
        color: Srgba,
    },
    /// A texture region.
    Image {
        part: crate::dress::PartRef,
        frame: Frame,
        texture: TextureHandle,
        uv: [f32; 4],
        color: Srgba,
    },
    /// One line of text centred in the frame, drawn only inside `clip`
    /// (`Frame::FULL`: the whole element; a meter's label is two of these,
    /// one ink over the fill and one over the track).
    Text {
        part: crate::dress::PartRef,
        frame: Frame,
        text: String,
        step: TypeStep,
        color: Srgba,
        font: FontRole,
        clip: Frame,
    },
    /// An icon of an icon set (an MTSDF font, `IconAtlas::font`): the
    /// glyph's em square fitted to the frame's shorter side, centred.
    Glyph {
        part: crate::dress::PartRef,
        frame: Frame,
        font: og_render::FontHandle,
        ch: char,
        color: Srgba,
    },
    /// A piece of the widget that is a part of its own (a meter's fill, a
    /// checkbox's box), drawn as the style dresses it, or from the tokens,
    /// with the widget's colour for it.
    Part {
        frame: Frame,
        part: crate::dress::PartRef,
        fill: Option<Srgba>,
    },
}

/// Which font a text uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontRole {
    #[default]
    Body,
    /// Display titles and key numbers (falls back to `Body`).
    Display,
    /// Code, ids and prices (falls back to `Body`).
    Mono,
    /// Levels and scores (falls back
    /// to `Display`, then `Body`).
    Rounded,
    /// A family a document's `@font-face` rules name (`font-family:
    /// "Name", <generic>`): its faces matched by weight, style and
    /// `unicode-range`, then its fallback role ([`crate::Fonts::families`]).
    Family(Family),
}

/// A named font family as a text refers to it: the family's id and the
/// role a text falls back to where the family has no face for a character
/// (the list's generic family; body when it names none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Family {
    id: u32,
    fallback: u8,
}

impl Family {
    /// `name` (compared case-insensitively, as CSS family names are)
    /// falling back to `fallback` (a role; a family's own fallback when
    /// `fallback` is a family).
    pub fn new(name: &str, fallback: FontRole) -> Family {
        Family {
            id: Family::id_of(name),
            fallback: match fallback.base() {
                FontRole::Display => 1,
                FontRole::Mono => 2,
                FontRole::Rounded => 3,
                _ => 0,
            },
        }
    }

    /// The id of the family `name`: FNV-1a over its ASCII-lowercased bytes.
    pub fn id_of(name: &str) -> u32 {
        name.bytes().fold(0x811c_9dc5, |h, b| {
            (h ^ u32::from(b.to_ascii_lowercase())).wrapping_mul(0x0100_0193)
        })
    }

    pub fn id(self) -> u32 {
        self.id
    }

    /// The four roles, in the order a family's fallback indexes them.
    pub const ROLES: [FontRole; 4] = [
        FontRole::Body,
        FontRole::Display,
        FontRole::Mono,
        FontRole::Rounded,
    ];

    /// The role a character the family has no face for is drawn in.
    pub fn fallback(self) -> FontRole {
        Family::ROLES[self.fallback_index()]
    }

    /// [`Family::fallback`]'s index in [`Family::ROLES`].
    pub fn fallback_index(self) -> usize {
        usize::from(self.fallback)
    }
}

impl FontRole {
    /// The role a style's text effects are keyed by (`body`, `display`,
    /// `numeric`): a rounded text takes the display role's.
    pub fn effect_index(self) -> usize {
        match self.base() {
            FontRole::Display | FontRole::Rounded => 1,
            FontRole::Mono => 2,
            _ => 0,
        }
    }

    /// The role itself, or a family's fallback role.
    pub fn base(self) -> FontRole {
        match self {
            FontRole::Family(f) => f.fallback(),
            r => r,
        }
    }

    /// The role an IDL name names (`body`, `display`, `mono`, `rounded`).
    pub fn from_name(name: &str) -> Option<FontRole> {
        Some(match name {
            "body" => FontRole::Body,
            "display" => FontRole::Display,
            "mono" => FontRole::Mono,
            "rounded" => FontRole::Rounded,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextContent {
    pub text: String,
    pub step: TypeStep,
    pub color: Srgba,
    pub align: TextAlign,
    pub wrap: bool,
    pub font: FontRole,
    /// Cut with "…" where it does not fit: a one-line text at its width, a
    /// wrapping one at its last line (`max_lines`).
    pub ellipsis: bool,
    /// The most lines a wrapping text shows (0: no limit).
    pub max_lines: u32,
    /// Extra faux-bold weight, em, over the style's own (a selected tab's
    /// label reads semibold); 0 for most text.
    pub emphasis: f32,
    /// Auto-fit: its own step, then the smaller steps it takes where its
    /// text does not fit its width, largest first (layout sets `step` to the
    /// first that fits, else the last); empty: it keeps its step.
    pub fit: Vec<TypeStep>,
    /// How its lines are chosen when it wraps (CSS `text-wrap-style`).
    pub wrap_style: WrapStyle,
    /// What its cut ends in instead of "…" (CSS `text-overflow: <string>`).
    pub cut: Option<String>,
    /// Set in a vertical line (CSS `writing-mode`): its widths are its
    /// advances down the line ([`crate::text::vertical`]). `None`:
    /// horizontal.
    pub vertical: Option<crate::layout::flow::VerticalText>,
}

/// CSS `text-wrap-style` (CSS Text 4 §6.2): `stable` lays out as `auto`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WrapStyle {
    #[default]
    Auto,
    /// Lines as even as the line count allows.
    Balance,
    /// No last line of one word where a narrower break avoids it.
    Pretty,
}

/// The OpenType features a text in `step` and `font` is set with: the
/// step's, and in the numeric font role tabular lining figures too, so a
/// score or a timer holds its width as it ticks (the typography audit,
/// fix 4).
pub(crate) fn text_features(step: &TypeStep, font: FontRole) -> crate::shape::Features {
    match font {
        FontRole::Mono => step.features.with(crate::shape::Features::FIGURES),
        _ => step.features,
    }
}

/// Text in a pixel font set on its grid (the typography audit, fix 5): its
/// size at a whole multiple of the grid and its tracking in whole pixels,
/// so each of the font's pixels is a whole number of CSS pixels and lands
/// on the device-pixel grid the pen is snapped to. A step's line height is
/// kept. Text in an outline font is unchanged.
pub(crate) fn snap_pixel_text(nodes: &mut [Node], fonts: &dyn Metrics) {
    let snap = |s: &mut TypeStep, m: &FontMetrics| {
        s.size = m.pixel_size(s.size);
        s.tracking_em = m.pixel_tracking(s.tracking_em);
        s.cjk_tracking_em = m.pixel_tracking(s.cjk_tracking_em);
    };
    for n in nodes {
        match &mut n.content {
            Content::Text(t) => {
                let m = t.metrics(fonts);
                if m.pixel_grid() > 0 {
                    snap(&mut t.step, m);
                    for s in &mut t.fit {
                        snap(s, m);
                    }
                }
            }
            Content::Rich(r) => {
                for s in r.spans.iter_mut().filter(|s| s.atom.is_none()) {
                    let m = fonts.metrics_styled(s.font, s.weight, s.italic, &s.text);
                    if m.pixel_grid() > 0 {
                        snap(&mut s.step, m);
                    }
                }
            }
            _ => {}
        }
    }
}

impl TextContent {
    /// The metrics of the face it is drawn in: its role's nearest its
    /// step's weight ([`crate::Fonts::for_styled`]).
    pub(crate) fn metrics<'a>(&self, fonts: &'a dyn Metrics) -> &'a FontMetrics {
        fonts.metrics_styled(self.font, self.step.weight, false, &self.text)
    }

    /// The OpenType features the text is set with ([`text_features`]).
    pub fn features(&self) -> crate::shape::Features {
        text_features(&self.step, self.font)
    }

    /// The type step's tracking, in em: the CJK tracking for text with CJK
    /// characters (0 unless a skin sets it), none for a shaped script.
    pub fn tracking(&self) -> f32 {
        step_tracking(&self.step, &self.text)
    }

    pub fn line_height(&self) -> f32 {
        if self.text.chars().any(is_cjk) {
            self.step.cjk_line_height
        } else {
            self.step.line_height
        }
    }
}

/// The ellipsis a font draws: "…" when it has the character, else "...".
pub(crate) fn ellipsis_of(m: &FontMetrics) -> &'static str {
    if m.has('…') { "…" } else { "..." }
}

/// Ellipsis truncation: lines past `max_lines` go, and the last line kept
/// (or a one-line text) wider than `width` loses characters from its end
/// until it fits with the ellipsis after it (a cut between words drops the
/// space before the ellipsis).
pub(super) fn truncate_lines(lines: &mut Vec<Line>, t: &TextContent, m: &FontMetrics, width: f32) {
    let (size, tracking) = (t.step.size, t.tracking());
    let cut = t.max_lines > 0 && lines.len() > t.max_lines as usize;
    if cut {
        lines.truncate(t.max_lines as usize);
    }
    let Some(last) = lines.last_mut() else {
        return;
    };
    if !cut && last.width <= width {
        return;
    }
    let dots = t.cut.as_deref().unwrap_or_else(|| ellipsis_of(m));
    let mut chars: Vec<char> = last.text.chars().collect();
    loop {
        let body: String = chars.iter().collect();
        let text = format!("{}{dots}", body.trim_end());
        let w = m.measure_with(&text, size, tracking, t.features());
        if w <= width || chars.is_empty() {
            last.text = text;
            last.width = w;
            return;
        }
        chars.pop();
    }
}

/// A step's tracking for `text`: none with a shaped script (its joins would break), the
/// step's CJK tracking with CJK characters (0 unless a skin's ramp sets it).
pub(crate) fn step_tracking(step: &TypeStep, text: &str) -> f32 {
    if text.chars().any(og_i18n::needs_shaping) {
        0.0
    } else if text.chars().any(is_cjk) {
        step.cjk_tracking_em
    } else {
        step.tracking_em
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Content {
    None,
    Text(TextContent),
    /// A paragraph of rich text spans (`crate::rich`).
    Rich(crate::rich::RichContent),
    Image {
        texture: Option<TextureHandle>,
        uv: [f32; 4],
        tint: Srgba,
        /// How it fills its box (CSS `object-fit`, `object-position`,
        /// `image-rendering`); stretched by default.
        object: ObjectFit,
    },
}

/// How an image element's texture fills its box: CSS `object-fit` (`fit`,
/// with `None` for scale-down's own size resolved by the renderer),
/// `object-position` (`focal`, the fraction of the free space, and
/// `offset`, the px after it) and `image-rendering` (`nearest`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectFit {
    pub fit: og_render::UiFit,
    pub focal: [f32; 2],
    pub offset: [f32; 2],
    pub nearest: bool,
}

impl Default for ObjectFit {
    fn default() -> Self {
        ObjectFit {
            fit: og_render::UiFit::Fill,
            focal: [0.5; 2],
            offset: [0.0; 2],
            nearest: false,
        }
    }
}

/// What a widget draws besides its background and content.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Widget {
    None,
    Button {
        pressed: bool,
        /// How far in the press is on the style's spring (0 rest, 1 held).
        press: f32,
    },
    Progress {
        value: f32,
    },
    Field(EditView),
    /// A multi-line text area: its text (the value with any composition),
    /// wrapped at draw time, and its placeholder.
    TextArea {
        edit: EditView,
        text: String,
        placeholder: String,
        line_height: f32,
        /// Its text's step and inks, as its style dresses them.
        ink: crate::ui::FieldInk,
    },
    Tooltip,
    /// A virtualised list or grid's filler: no drawing.
    Spacer,
}

/// A focused text control's caret, selection and composition, in
/// characters of the text it shows, and its scroll.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct EditView {
    pub focused: bool,
    pub caret: usize,
    /// The selection `[start, end)` (empty: none).
    pub selection: (usize, usize),
    /// The composition's range, drawn underlined.
    pub composition: Option<(usize, usize)>,
    pub scroll: f32,
    /// The caret shows this tick (its blink).
    pub caret_on: bool,
    /// The caret is drawn after the glyph before it (`caret_x_at`).
    pub upstream: bool,
    /// The control shows its placeholder (its value is empty).
    pub placeholder: bool,
}

pub(crate) const NONE: usize = usize::MAX;

/// A staggered entrance: descendant k starts `k × step` ticks after `since`
/// and takes `ticks`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Stagger {
    pub since: u64,
    pub step: f32,
    pub ticks: f32,
    pub kind: crate::motion::TransitionKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SampledStyle {
    pub transform: crate::element::Transform,
    pub opacity: f32,
    pub radius: f32,
    pub background: Option<Srgba>,
    pub ink: Srgba,
    pub z: i32,
    pub inert: bool,
    /// [`crate::element::Prop::Blur`] and [`crate::element::Prop::Draw`]:
    /// 0 but while a transition moves them.
    pub blur: f32,
    pub draw: f32,
}

impl Default for SampledStyle {
    fn default() -> Self {
        Self {
            transform: crate::element::Transform::IDENTITY,
            opacity: 1.0,
            radius: 0.0,
            background: None,
            ink: Srgba::WHITE,
            z: 0,
            inert: false,
            blur: 0.0,
            draw: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub key: u64,
    pub id: String,
    pub parent: usize,
    pub children: Vec<usize>,
    /// Its children as built or reattached, hidden ones included: what the
    /// frame derives `children`, `ordered_children` and `out` from, so a
    /// retained node that is not built again derives the same lists.
    pub built_children: Vec<usize>,
    /// Sibling order before the flow/out-of-flow partition, independent of slots.
    pub ordered_children: Vec<usize>,
    /// The scroll gutter added to its right padding (`Ui::apply_gutter`).
    pub gutter: f32,
    /// Its authored width, height and padding while placement replaced
    /// them for one frame (`Ui::layout_mutated`).
    pub authored_box: Option<(Size, Size, Insets)>,
    pub el: Element,
    pub content: Content,
    pub widget: Widget,
    /// The text the probe reports for it.
    pub probe_text: String,
    /// A disabled control (the probe reports it; contrast rules exempt it).
    pub disabled: bool,
    pub focusable: bool,
    /// A multi-line text control: ↑ and ↓ belong to it while it is edited.
    pub multiline: bool,
    /// For a floating element: the node it is placed against.
    pub anchor: usize,
    /// How it is placed when it is a layer of its own.
    pub float: Float,
    /// A transition's effect on its subtree.
    pub fx: Option<Fx>,
    /// Keyboard and gamepad focus can land on it.
    pub nav: bool,
    /// Left and right adjust it when focused (slider, stepper).
    pub adjust: bool,
    /// The arrows it claims while focused ([`crate::ARROW_LEFT`] …).
    pub claim: u8,
    /// Draw the focus ring (focused by keyboard or gamepad).
    pub ring: bool,
    /// Bit k: pad k + 1's focus is on it (its ring in that player's colour).
    pub pad_rings: u8,
    /// Built but not shown (a closed dialog's slot): no layout, drawing or input.
    pub hidden: bool,
    /// A modal backdrop: takes the pointer, never focus.
    pub backdrop: bool,
    /// A focus scope (dialog, open menu): focus stays inside while it is the topmost.
    pub scope: bool,
    /// A scope Back does not close (a locked dialog): while it is the
    /// topmost, Back reaches `Ui::back`.
    pub locked: bool,
    /// Drawn after the background.
    pub shapes: Vec<Shape>,
    /// A document's CSS border, outline, shadows and clip
    /// ([`crate::decor::BoxDecor`]), painted over its draw.
    pub decor: Option<Box<crate::decor::BoxDecor>>,
    /// Widget data its children read while they are built (a list's row
    /// height, a grid's cell size, the filler after the rows in view);
    /// never drawn, so a change to it repaints nothing.
    pub aux: [f32; 3],
    /// A heading label: the style's flourish frames it.
    pub heading: bool,
    /// What it tells the accessibility tree beyond its kind.
    pub a11y: crate::a11y::A11y,
    /// Its descendants enter one after another (a style's stagger).
    pub stagger: Option<Stagger>,
    /// The element model beyond its box (`crate::element`).
    pub props: crate::element::Props,
    /// Frame sampling never changes the authored element, content or props.
    pub sampled: SampledStyle,
    /// CSS `accent-color` and `caret-color` it declares or inherits as a
    /// document element (controls under it read the nearest, [`crate::Ui`]).
    pub accent: Option<Srgba>,
    pub caret: Option<Srgba>,
    /// Its children out of the flow (absolute, anchored), in build order;
    /// `children` holds the ones in the flow once the frame ends, in CSS
    /// `order`.
    pub out: Vec<usize>,
    /// One of its children in the flow aligns by its own baseline
    /// (`align-self: baseline`).
    pub baseline_items: bool,
    /// The node it lays out in: its parent, else the box its parent's
    /// `display: contents` (or a table's rows) hands it to.
    pub up: usize,
    /// A table's cells and the boxes of its rows, groups and columns.
    pub table: Option<crate::layout::Table>,
    /// It holds inline-level children and lays them out in lines (CSS 2
    /// §9.4.2): its authored direction, wrap, cross and align, kept to put
    /// back when it no longer does.
    pub line_box: Option<(Direction, bool, Cross, Align)>,
    /// Its content's least width and height, padding included, before its
    /// own size applies (a flex item's automatic minimum, CSS Flexbox §4.5).
    pub min_content: [f32; 2],
    /// A grid's resolved tracks and its children's cells.
    pub grid: Option<GridLayout>,
    /// Where it is drawn: its map from layout to screen (its transforms and
    /// its ancestors'), and its box's bounds there.
    pub world: crate::element::Affine,
    pub screen: Rect,
    /// Its place in a 3D space (`None`: drawn flat by `world`).
    pub plane: Option<Box<crate::transform3d::Plane>>,
    /// Layout animation's offset and scale this tick (x, y, sx, sy).
    pub flip: [f32; 4],
    /// A drag's offset and a pinch's scale and turn (x, y, scale, radians).
    pub gesture: [f32; 4],
    /// The player's HUD placement (`crate::hud_layout`): offset, scale and
    /// opacity (x, y, scale, opacity).
    pub custom: [f32; 4],
    /// Its view-box fit ([`crate::element::ViewPlace`]), written by layout
    /// where it places it: the scale, the anchor across its box (a
    /// fraction) and down it (CSS px), and its anchor's offset across from
    /// its parent's centre (right to left the mirror flips that offset,
    /// and the draw moves it back where its view box shows it).
    pub fit: [f32; 4],
    // Computed by the frame.
    pub alpha: f32,
    pub scale: f32,
    pub inert: bool,
    /// The scroll container whose clip applies (NONE: the screen).
    pub clip_src: usize,
    // Computed.
    pub pref_w: f32,
    pub min_w: f32,
    pub pref_h: f32,
    pub min_h: f32,
    pub w: f32,
    pub h: f32,
    pub x: f32,
    pub y: f32,
    pub content_h: f32,
    /// Its outer top and bottom margins after collapsing (CSS 2 §8.3.1):
    /// each the largest positive and the most negative margin of the set
    /// that adjoins that edge, its own and those collapsed through it.
    pub margin_sets: [(f32, f32); 2],
    /// Its first child's top and its last child's bottom margin collapse
    /// through its own edge: inside, that edge has none.
    pub collapse_in: [bool; 2],
    /// An empty block in a block's flow: its top and bottom margins adjoin
    /// and collapse together with its neighbours' (CSS 2 §8.3.1).
    pub through: bool,
    /// Its left and right margins as laid out, `auto` ones resolved.
    pub margin_x: [f32; 2],
    /// The content's width (its children's extent), for horizontal scrolling.
    pub content_w: f32,
    pub lines: Vec<Line>,
    /// The characters of its text the player selected (`user-select`),
    /// `[start, end)` over its lines joined by one break each.
    pub selected: Option<(u32, u32)>,
    /// A rich text's lines.
    pub rich_lines: Vec<crate::rich::RichLine>,
    /// A rich text's links, by id, in order.
    pub links: Vec<String>,
    /// A wrapping row's lines: ranges of `children`.
    pub rows: Vec<(usize, usize)>,
    pub clip: Rect,
    pub rect: Rect,
    /// Constraints and parent-relative position of the cached layout.
    pub layout_size: [f32; 2],
    pub layout_offset: [f32; 2],
    pub layout_step: Option<TypeStep>,
    /// What layout last gave it before the frame moved it (a right-to-left
    /// mirror, a transformed scroller's clip): its place, its box (with a
    /// baseline or anchor shift) and its clip.
    pub laid_xy: [f32; 2],
    pub laid_rect: Rect,
    pub laid_clip: Rect,
    /// Where anchor positioning put it this frame (CSS Anchor
    /// Positioning 1): the position option that won, the side of its
    /// anchor it sits on, and whether `position-visibility` hides it.
    pub anchor_placed: crate::anchor::Placed,
    /// A multi-column container's used column count and width.
    pub cols: (usize, f32),
    /// A multi-column container's rules between columns, from its box's
    /// top left: each one's middle x, its top and its length.
    pub col_rules: Vec<[f32; 3]>,
    /// A text a column break splits: from each line on, how far its lines
    /// move from where its box puts them (into the next column).
    pub frag: Vec<(u32, [f32; 2])>,
    /// A sticky box's edges of its scrollport it is held to this frame
    /// (bits of `doc::media::scroll::TOP` …).
    pub stuck: u8,
    /// It skips its contents this frame (`content-visibility: hidden`, or
    /// `auto` far from the screen): they are not laid out, drawn or hit.
    pub cv_skip: bool,
    /// A `content-visibility: auto` element was far from the screen when
    /// last laid out (it skips its contents from the next frame).
    pub cv_far: bool,
    /// Its content box's size when it last laid its contents out, for
    /// `contain-intrinsic-size: auto` (CSS Sizing 4 §5.1).
    pub remembered: Option<[f32; 2]>,
    /// Its floated children (CSS 2 §9.5): out of the flow, in
    /// `ordered_children`, placed by [`crate::layout::float`].
    pub floats: Vec<usize>,
    /// A float's border box from its parent's content box.
    pub float_at: [f32; 2],
    /// The room put above it in its parent's flow to clear floats (or to
    /// move a box that does not overlap floats below them).
    pub clearance: f32,
    /// A text whose lines are shortened around floats: per line, its
    /// shift right and down from where its box puts it and the width its
    /// line box has.
    pub wrap_lines: Vec<[f32; 3]>,
    /// A block formatting context root: the bottom of its floats below its
    /// content box's top (its height holds them).
    pub float_h: f32,
    /// Floats narrowed or moved it, or shortened its lines, last layout.
    pub beside: bool,
    /// A block laying out lines that hold a box with a `vertical-align`:
    /// per line, its height as its line box holds them (CSS 2.1 §10.8);
    /// `None` (or empty): the line is its tallest child's height.
    pub inline_rows: Vec<Option<f32>>,
    /// Its border box's top below its line's top, in such a line.
    pub inline_y: f32,
}

impl Node {
    pub(crate) fn sample_style(&mut self) {
        self.sampled = SampledStyle {
            transform: self.props.transform,
            opacity: self.props.opacity,
            radius: self.el.radius,
            background: self.el.background,
            ink: match &self.content {
                Content::Text(text) => text.color,
                _ => Srgba::WHITE,
            },
            z: self.props.z,
            inert: self.props.inert,
            blur: 0.0,
            draw: 0.0,
        };
    }

    pub fn new(key: u64, id: String, parent: usize, el: Element) -> Node {
        Node {
            key,
            id,
            parent,
            children: Vec::new(),
            built_children: Vec::new(),
            baseline_items: false,
            up: parent,
            table: None,
            line_box: None,
            min_content: [0.0; 2],
            ordered_children: Vec::new(),
            gutter: 0.0,
            authored_box: None,
            el,
            content: Content::None,
            widget: Widget::None,
            probe_text: String::new(),
            disabled: false,
            focusable: false,
            multiline: false,
            anchor: NONE,
            float: Float::None,
            fx: None,
            nav: false,
            adjust: false,
            claim: 0,
            ring: false,
            pad_rings: 0,
            hidden: false,
            backdrop: false,
            scope: false,
            locked: false,
            shapes: Vec::new(),
            decor: None,
            aux: [0.0; 3],
            heading: false,
            a11y: crate::a11y::A11y::default(),
            stagger: None,
            props: crate::element::Props::default(),
            sampled: SampledStyle::default(),
            accent: None,
            caret: None,
            out: Vec::new(),
            grid: None,
            plane: None,
            world: crate::element::Affine::IDENTITY,
            screen: Rect::default(),
            flip: [0.0, 0.0, 1.0, 1.0],
            gesture: [0.0, 0.0, 1.0, 0.0],
            custom: [0.0, 0.0, 1.0, 1.0],
            fit: [1.0, 0.0, 0.0, 0.0],
            alpha: 1.0,
            scale: 1.0,
            inert: false,
            clip_src: NONE,
            pref_w: 0.0,
            min_w: 0.0,
            pref_h: 0.0,
            min_h: 0.0,
            w: 0.0,
            h: 0.0,
            x: 0.0,
            y: 0.0,
            content_h: 0.0,
            content_w: 0.0,
            lines: Vec::new(),
            selected: None,
            rich_lines: Vec::new(),
            links: Vec::new(),
            rows: Vec::new(),
            clip: Rect::default(),
            rect: Rect::default(),
            layout_size: [0.0; 2],
            layout_offset: [0.0; 2],
            layout_step: None,
            margin_sets: [(0.0, 0.0); 2],
            collapse_in: [false; 2],
            through: false,
            margin_x: [0.0; 2],
            laid_xy: [f32::NAN; 2],
            laid_rect: Rect::default(),
            laid_clip: Rect::default(),
            anchor_placed: crate::anchor::Placed::default(),
            cols: (1, 0.0),
            col_rules: Vec::new(),
            frag: Vec::new(),
            stuck: 0,
            cv_skip: false,
            cv_far: false,
            remembered: None,
            floats: Vec::new(),
            float_at: [0.0; 2],
            clearance: 0.0,
            wrap_lines: Vec::new(),
            float_h: 0.0,
            beside: false,
            inline_rows: Vec::new(),
            inline_y: 0.0,
        }
    }

    /// Immediate builders import their prior layout once. Retained nodes
    /// already own these outputs; layout never needs a second tree.
    pub(crate) fn inherit_layout(&mut self, previous: &Self, clean: bool) {
        self.w = previous.w;
        self.h = previous.h;
        self.x = previous.x;
        self.y = previous.y;
        self.layout_size = previous.layout_size;
        self.layout_offset = previous.layout_offset;
        self.fit = previous.fit;
        self.laid_xy = previous.laid_xy;
        self.laid_rect = previous.laid_rect;
        self.anchor_placed = previous.anchor_placed;
        self.cols = previous.cols;
        self.col_rules.clone_from(&previous.col_rules);
        self.frag.clone_from(&previous.frag);
        self.stuck = previous.stuck;
        self.laid_clip = previous.laid_clip;
        self.float_at = previous.float_at;
        self.inline_y = previous.inline_y;
        if !clean {
            return;
        }
        self.pref_w = previous.pref_w;
        self.min_w = previous.min_w;
        self.pref_h = previous.pref_h;
        self.min_h = previous.min_h;
        self.content_w = previous.content_w;
        self.content_h = previous.content_h;
        self.lines.clone_from(&previous.lines);
        self.wrap_lines.clone_from(&previous.wrap_lines);
        self.clearance = previous.clearance;
        self.float_h = previous.float_h;
        self.beside = previous.beside;
        self.inline_rows.clone_from(&previous.inline_rows);
        self.rich_lines.clone_from(&previous.rich_lines);
        self.rows.clone_from(&previous.rows);
        self.grid.clone_from(&previous.grid);
        self.layout_step = previous.layout_step;
        self.margin_sets = previous.margin_sets;
        self.collapse_in = previous.collapse_in;
        self.through = previous.through;
        self.margin_x = previous.margin_x;
    }

    /// Its content size along `axis` (0 across) under size containment:
    /// its `contain-intrinsic-*` length, its last remembered size for
    /// `auto` while it skips its contents, else 0 (CSS Containment 2
    /// §3.1, CSS Sizing 4 §5). `None` without size containment there.
    pub(crate) fn contained_size(&self, axis: usize) -> Option<f32> {
        let c = &self.el.contain;
        let bit = [
            crate::layout::Contain::SIZE_W,
            crate::layout::Contain::SIZE_H,
        ][axis];
        if !self.cv_skip && !c.has(bit) {
            return None;
        }
        let (auto, len) = c.intrinsic[axis];
        Some(match self.remembered {
            Some(r) if auto && self.cv_skip => r[axis],
            _ => len.unwrap_or(0.0),
        })
    }

    /// How far a column break moved line `k` of its text ([`Node::frag`]).
    pub(crate) fn frag_at(&self, k: usize) -> [f32; 2] {
        self.frag
            .iter()
            .rev()
            .find(|f| f.0 as usize <= k)
            .map_or([0.0; 2], |f| f.1)
    }

    /// The scroll range: how far the content overflows the inner height.
    pub fn max_scroll(&self) -> f32 {
        if !self.el.scroll {
            return 0.0;
        }
        (self.content_h - (self.h - self.el.padding.vertical())).max(0.0)
    }

    /// The horizontal scroll range: how far the content overflows the inner width.
    pub fn max_scroll_x(&self) -> f32 {
        if !self.el.scroll_x {
            return 0.0;
        }
        (self.content_w - (self.w - self.el.padding.horizontal())).max(0.0)
    }

    /// Scrolls either way.
    pub fn scrolls(&self) -> bool {
        self.el.scroll || self.el.scroll_x
    }
}
