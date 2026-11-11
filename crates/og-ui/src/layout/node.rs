use super::*;

mod authored;
pub(crate) use authored::Authored;

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
        font: og_draw::FontHandle,
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
    /// text does not fit its width, largest first (layout chooses the first
    /// that fits, else the last: [`LayoutBox::fit_step`]); empty: it keeps
    /// its step.
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
                let m = t.authored().metrics(fonts);
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
    /// The text set at `step`: one of the steps an auto-fit text tries, or
    /// the one layout chose for it.
    pub(crate) fn at(&self, step: TypeStep) -> UsedText<'_> {
        UsedText {
            content: self,
            step,
        }
    }

    /// The text at the step it was written with: what builds it measures
    /// it so before it is laid out.
    pub(crate) fn authored(&self) -> UsedText<'_> {
        self.at(self.step)
    }
}

/// A text as layout and paint read it: its content (it derefs to it) at
/// the step it is set at, which for an auto-fit text is the one layout
/// chose ([`LayoutBox::fit_step`]), not the one it was written with.
/// [`Node::text`] makes a node's.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UsedText<'a> {
    content: &'a TextContent,
    pub step: TypeStep,
}

impl std::ops::Deref for UsedText<'_> {
    type Target = TextContent;

    fn deref(&self) -> &TextContent {
        self.content
    }
}

impl<'a> UsedText<'a> {
    /// The metrics of the face it is drawn in: its role's nearest its
    /// step's weight ([`crate::Fonts::for_styled`]).
    pub(crate) fn metrics<'m>(&self, fonts: &'m dyn Metrics) -> &'m FontMetrics {
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

    /// The content it reads, for as long as the node holds it.
    pub(crate) fn content(&self) -> &'a TextContent {
        self.content
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
pub(super) fn truncate_lines(lines: &mut Vec<Line>, t: UsedText<'_>, m: &FontMetrics, width: f32) {
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
    pub fit: og_draw::UiFit,
    pub focal: [f32; 2],
    pub offset: [f32; 2],
    pub nearest: bool,
}

impl Default for ObjectFit {
    fn default() -> Self {
        ObjectFit {
            fit: og_draw::UiFit::Fill,
            focal: [0.5; 2],
            offset: [0.0; 2],
            nearest: false,
        }
    }
}

/// Widget geometry derived from measured persistent parts before paint.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PartGeometry {
    Switch {
        track: u64,
        knob: u64,
        mark: u64,
        tick: u64,
        measured: Option<[f32; 4]>,
    },
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
pub(crate) struct PartBox {
    pub owner: usize,
    pub reference: crate::dress::PartRef,
    pub element: Element,
    pub intrinsic_props: crate::element::Props,
}

/// Where the frame places a node: its map to the screen, its bounds,
/// clip and visibility there. Layout writes it; paint, hits and the
/// probe read it.
#[derive(Clone, Debug)]
pub(crate) struct Placed {
    /// Where it is drawn: its map from layout to screen (its transforms and
    /// its ancestors'), and its box's bounds there.
    pub world: crate::element::Affine,
    pub screen: Rect,
    /// Its place in a 3D space (`None`: drawn flat by `world`).
    pub plane: Option<Box<crate::transform3d::Plane>>,
    // Computed by the frame.
    pub alpha: f32,
    pub scale: f32,
    pub inert: bool,
    /// The scroll container whose clip applies (NONE: the screen).
    pub clip_src: usize,
    pub clip: Rect,
    pub rect: Rect,
}

impl Default for Placed {
    fn default() -> Self {
        Self {
            plane: None,
            world: crate::element::Affine::IDENTITY,
            screen: Rect::default(),
            alpha: 1.0,
            scale: 1.0,
            inert: false,
            clip_src: NONE,
            clip: Rect::default(),
            rect: Rect::default(),
        }
    }
}

/// The box placement fixes for a node after it measured it: its size and
/// the padding it adds (the safe area under a modal's backdrop).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement {
    pub size: [f32; 2],
    pub padding: Insets,
}

/// What a breakpoint or orientation variant's query reads of the viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Media {
    pub breakpoint: crate::style::Breakpoint,
    pub portrait: bool,
    pub width: f32,
}

impl Media {
    /// Whether a variant's query (not a container's) holds.
    pub(crate) fn matches(&self, q: &crate::element::Query) -> bool {
        (q.breakpoints.is_empty() || q.breakpoints.contains(&self.breakpoint))
            && q.orientation
                .is_none_or(|o| (o == crate::element::Orientation::Portrait) == self.portrait)
            && crate::layout::width_in(self.width, q)
    }
}

/// What the last write of a node's input changed against the node it
/// replaced, recorded when the build ends ([`crate::layout::input_changes`]):
/// what the frame decides reuse by, in place of a copy of the last frame's
/// node. Read only for a node written this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Changed(u8);

impl Changed {
    /// Nothing was there to compare with: a new node.
    pub const NEW: Changed = Changed(1);
    /// Any of its inputs (`LayoutStats::rebuilt` counts it).
    pub const INPUT: Changed = Changed(2);
    /// What layout reads of it: its box, content, placement, lists.
    pub const LAYOUT: Changed = Changed(4);
    /// What a fixed-size box still gives its parent: its size, margins,
    /// place and whether a margin collapsed through it.
    pub const BOX: Changed = Changed(8);
    /// Whether it is a block or how it arranges its children: their
    /// margins collapse otherwise.
    pub const FLOW: Changed = Changed(16);

    /// Whether any of `bits` is set.
    pub(crate) fn has(self, bits: Changed) -> bool {
        self.0 & bits.0 != 0
    }
}

impl std::ops::BitOr for Changed {
    type Output = Changed;
    fn bitor(self, o: Changed) -> Changed {
        Changed(self.0 | o.0)
    }
}

impl std::ops::BitOrAssign for Changed {
    fn bitor_assign(&mut self, o: Changed) {
        self.0 |= o.0;
    }
}

/// What layout computes for a node: its sizes, its place in its parent,
/// its lines, rows, tracks and floats, and what the layout cache keeps.
#[derive(Clone, Debug)]
pub(crate) struct LayoutBox {
    /// The element layout lays it out with, and what paint, scrolling, hit
    /// testing and inspection read of its box: its authored element
    /// ([`Node::el`]) with what the frame decides on top of it
    /// ([`Node::derive_used`]). Layout writes this, never the authored one.
    pub used: Element,
    /// Its insets as layout reads them (a vertical frame turns them).
    pub insets: crate::element::Edges,
    /// The room its scroll bar takes beside its content, added to its
    /// used padding (`Layout::apply_gutter`).
    pub gutter: f32,
    /// The box placement gave it this frame: a layer's, a screen-placed or
    /// anchor-positioned element's (`Layout::placed`).
    pub placement: Option<Placement>,
    /// The viewport its breakpoint and orientation variants were matched
    /// against when it was written (`None`: it has none).
    pub media: Option<Media>,
    /// The width and height the user gave it by its resize handle (CSS
    /// `resize`), as of when it was written (`Tree::apply_resized`).
    pub resized: [Option<f32>; 2],
    /// The inline size its container variants were last matched at.
    pub query_w: Option<f32>,
    /// One of its children in the flow aligns by its own baseline
    /// (`align-self: baseline`).
    pub baseline_items: bool,
    /// A table's cells and the boxes of its rows, groups and columns.
    pub table: Option<crate::layout::Table>,
    /// It holds inline-level children and lays them out in lines (CSS 2
    /// §9.4.2), as a wrapping row: the direction it has as a block.
    pub line_box: Option<Direction>,
    /// Its content's least width and height, padding included, before its
    /// own size applies (a flex item's automatic minimum, CSS Flexbox §4.5).
    pub min_content: [f32; 2],
    /// A grid's resolved tracks and its children's cells.
    pub grid: Option<GridLayout>,
    /// Its view-box fit ([`crate::element::ViewPlace`]), written by layout
    /// where it places it: the scale, the anchor across its box (a
    /// fraction) and down it (CSS px), and its anchor's offset across from
    /// its parent's centre (right to left the mirror flips that offset,
    /// and the draw moves it back where its view box shows it).
    pub fit: [f32; 4],
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
    /// A rich text's lines.
    pub rich_lines: Vec<crate::rich::RichLine>,
    /// A rich text's links, by id, in order.
    pub links: Vec<String>,
    /// A wrapping row's lines: ranges of `children`.
    pub rows: Vec<(usize, usize)>,
    /// Constraints and parent-relative position of the cached layout.
    pub layout_size: [f32; 2],
    pub layout_offset: [f32; 2],
    /// An auto-fit text's step: the largest of its steps its text fits its
    /// width at, chosen where its lines are broken (`distribute_w`) and
    /// kept with them. `None`: the step it was written with.
    pub fit_step: Option<TypeStep>,
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

impl Default for LayoutBox {
    fn default() -> Self {
        Self {
            used: Element::default(),
            insets: crate::element::Edges::default(),
            gutter: 0.0,
            placement: None,
            media: None,
            resized: [None; 2],
            query_w: None,
            baseline_items: false,
            table: None,
            line_box: None,
            min_content: [0.0; 2],
            grid: None,
            fit: [1.0, 0.0, 0.0, 0.0],
            pref_w: 0.0,
            min_w: 0.0,
            pref_h: 0.0,
            min_h: 0.0,
            w: 0.0,
            h: 0.0,
            x: 0.0,
            y: 0.0,
            content_h: 0.0,
            margin_sets: [(0.0, 0.0); 2],
            collapse_in: [false; 2],
            through: false,
            margin_x: [0.0; 2],
            content_w: 0.0,
            lines: Vec::new(),
            rich_lines: Vec::new(),
            links: Vec::new(),
            rows: Vec::new(),
            layout_size: [0.0; 2],
            layout_offset: [0.0; 2],
            fit_step: None,
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
            float_at: [0.0; 2],
            clearance: 0.0,
            wrap_lines: Vec::new(),
            float_h: 0.0,
            beside: false,
            inline_rows: Vec::new(),
            inline_y: 0.0,
        }
    }
}

/// What the frame's style and motion sample for a node this tick: its
/// animated values, its layout animation, gesture and HUD offsets.
#[derive(Clone, Debug)]
pub(crate) struct StyleNode {
    /// Frame sampling never changes the authored element, content or props.
    pub sampled: SampledStyle,
    /// Layout animation's offset and scale this tick (x, y, sx, sy).
    pub flip: [f32; 4],
    /// A drag's offset and a pinch's scale and turn (x, y, scale, radians).
    pub gesture: [f32; 4],
    /// The player's HUD placement (`crate::hud_layout`): offset, scale and
    /// opacity (x, y, scale, opacity).
    pub custom: [f32; 4],
}

impl Default for StyleNode {
    fn default() -> Self {
        Self {
            sampled: SampledStyle::default(),
            flip: [0.0, 0.0, 1.0, 1.0],
            gesture: [0.0, 0.0, 1.0, 0.0],
            custom: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

/// What interaction marks on a node for paint: its focus rings and the
/// characters the player selected.
#[derive(Clone, Debug, Default)]
pub(crate) struct NodeMarks {
    /// Draw the focus ring (focused by keyboard or gamepad).
    pub ring: bool,
    /// Bit k: pad k + 1's focus is on it (its ring in that player's colour).
    pub pad_rings: u8,
    /// The characters of its text the player selected (`user-select`),
    /// `[start, end)` over its lines joined by one break each.
    pub selected: Option<(u32, u32)>,
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
    /// What layout computes and keeps for it ([`LayoutBox`]).
    pub layout: LayoutBox,
    /// What its last write changed (`Layout::reuse_check`).
    pub changed: Changed,
    /// Its element as authored: layout reads it and lays out
    /// [`LayoutBox::used`].
    pub el: Authored,
    pub part_host: Option<std::sync::Arc<crate::doc::build::PartHost>>,
    /// The semantic owner of a detached widget surface's CSS parts.
    /// Placement anchors alone do not imply shared CSS ownership.
    pub part_source: usize,
    pub part_box: Option<PartBox>,
    pub content_slot: usize,
    pub content: Content,
    pub widget: Widget,
    pub part_geometry: Option<PartGeometry>,
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
    /// A float's inline exclusion contour, independent of its margin box.
    pub shape_outside: Option<super::float::Outside>,
    /// A transition's effect on its subtree.
    pub fx: Option<Fx>,
    /// Keyboard and gamepad focus can land on it.
    pub nav: bool,
    /// Left and right adjust it when focused (slider, stepper).
    pub adjust: bool,
    /// The arrows it claims while focused ([`crate::ARROW_LEFT`] …).
    pub claim: u8,
    /// Its focus rings and selected characters ([`NodeMarks`]).
    pub marks: NodeMarks,
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
    /// What style and motion sample for it this tick ([`StyleNode`]).
    pub style: StyleNode,
    /// CSS `accent-color` and `caret-color` it declares or inherits as a
    /// document element (controls under it read the nearest, [`crate::Ui`]).
    pub accent: Option<Srgba>,
    pub caret: Option<Srgba>,
    /// Its children out of the flow (absolute, anchored), in build order;
    /// `children` holds the ones in the flow once the frame ends, in CSS
    /// `order`.
    pub out: Vec<usize>,
    /// The node it lays out in: its parent, else the box its parent's
    /// `display: contents` (or a table's rows) hands it to.
    pub up: usize,
    /// Where the frame places and draws it (layout writes it, paint reads it).
    pub placed: Placed,
    // Computed.
    /// A popover opened at a point (`contextmenu`): its implicit anchor is
    /// that zero-size point rather than its anchor node.
    pub anchor_point: Option<[f32; 2]>,
    /// A popover's `::-og-arrow`: its side and colour, drawn on the side of
    /// its box facing its anchor once it is placed.
    pub arrow: Option<(f32, Srgba)>,
    /// Its floated children (CSS 2 §9.5): out of the flow, in
    /// `ordered_children`, placed by [`crate::layout::float`].
    pub floats: Vec<usize>,
}

impl Node {
    /// Its rich text's vertical setting (its element's writing mode), `None`
    /// when its lines run across; a rich text combines nothing upright.
    pub(crate) fn rich_vertical(&self) -> Option<crate::layout::flow::VerticalText> {
        self.layout.used.vertical?;
        self.props
            .flow
            .vertical_text("")
            .map(|v| crate::layout::flow::VerticalText {
                combine: false,
                ..v
            })
    }

    /// Its text as layout and paint read it: at the step layout chose for
    /// an auto-fit text, else its own.
    pub(crate) fn text(&self) -> Option<UsedText<'_>> {
        match &self.content {
            Content::Text(t) => Some(self.used_text(t)),
            _ => None,
        }
    }

    /// [`Node::text`] of its content `t`, where that is already matched.
    pub(crate) fn used_text<'a>(&self, t: &'a TextContent) -> UsedText<'a> {
        t.at(self.layout.fit_step.unwrap_or(t.step))
    }

    /// The element layout uses, from the authored one: its breakpoint and
    /// orientation variants, the size the user resized it to, the line box
    /// it lays inline children out in, its scroll bar's gutter, with
    /// `placed` the box placement fixed, and its container variants at the
    /// size they were last matched at.
    pub(crate) fn derived(&self, placed: bool) -> Element {
        let mut u = *self.el;
        if let Some(m) = self.layout.media {
            for v in &self.props.variants {
                if !v.when.container && m.matches(&v.when) {
                    crate::layout::patch_box(&mut u, &v.set);
                }
            }
        }
        if let Some(w) = self.layout.resized[0] {
            u.width = Size::Fixed(w);
        }
        if let Some(h) = self.layout.resized[1] {
            u.height = Size::Fixed(h);
        }
        if self.layout.line_box.is_some() {
            u.direction = Direction::Row;
            u.wrap = true;
            u.cross = Cross::Baseline;
            u.align = u.line_align;
        }
        if self.layout.gutter != 0.0 {
            u.padding.right += self.layout.gutter;
            // `scrollbar-gutter: stable both-edges`: the same room at the start.
            if u.scroll_css.gutter == 2 {
                u.padding.left += self.layout.gutter;
            }
        }
        if let Some(p) = self.layout.placement.filter(|_| placed) {
            u.width = Size::Fixed(p.size[0]);
            u.height = Size::Fixed(p.size[1]);
            u.padding = u.padding.add(&p.padding);
        }
        if let Some(w) = self.layout.query_w {
            for v in &self.props.variants {
                if v.when.container && crate::layout::width_in(w, &v.when) {
                    crate::layout::patch_box(&mut u, &v.set);
                }
            }
        }
        u
    }

    /// Makes [`LayoutBox::used`] and its insets again from the authored
    /// element and props: after they were written, and wherever the frame
    /// changes what it decides on top of them.
    pub(crate) fn derive_used(&mut self) {
        self.layout.used = self.derived(true);
        self.layout.insets = self.props.insets;
    }

    pub(crate) fn sample_style(&mut self) {
        self.style.sampled = SampledStyle {
            transform: self.props.transform,
            opacity: self.props.opacity,
            radius: self.layout.used.radius,
            background: self.layout.used.background,
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
            layout: LayoutBox {
                used: el,
                ..LayoutBox::default()
            },
            up: parent,
            ordered_children: Vec::new(),
            changed: Changed::NEW,
            el: Authored::new(el),
            part_host: None,
            part_source: NONE,
            part_box: None,
            content_slot: NONE,
            content: Content::None,
            widget: Widget::None,
            part_geometry: None,
            probe_text: String::new(),
            disabled: false,
            focusable: false,
            multiline: false,
            anchor: NONE,
            float: Float::None,
            shape_outside: None,
            fx: None,
            nav: false,
            adjust: false,
            claim: 0,
            marks: NodeMarks::default(),
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
            style: StyleNode::default(),
            accent: None,
            caret: None,
            out: Vec::new(),
            placed: Placed::default(),
            anchor_point: None,
            arrow: None,
            floats: Vec::new(),
        }
    }

    /// Immediate builders import their prior layout once. Retained nodes
    /// already own these outputs; layout never needs a second tree.
    pub(crate) fn inherit_layout(&mut self, previous: &Self, clean: bool) {
        self.layout.w = previous.layout.w;
        self.layout.h = previous.layout.h;
        self.layout.x = previous.layout.x;
        self.layout.y = previous.layout.y;
        self.layout.layout_size = previous.layout.layout_size;
        self.layout.layout_offset = previous.layout.layout_offset;
        self.layout.fit = previous.layout.fit;
        self.layout.laid_xy = previous.layout.laid_xy;
        self.layout.laid_rect = previous.layout.laid_rect;
        self.layout.anchor_placed = previous.layout.anchor_placed;
        self.layout.cols = previous.layout.cols;
        self.layout.col_rules.clone_from(&previous.layout.col_rules);
        self.layout.frag.clone_from(&previous.layout.frag);
        self.layout.stuck = previous.layout.stuck;
        self.layout.laid_clip = previous.layout.laid_clip;
        self.layout.float_at = previous.layout.float_at;
        self.layout.inline_y = previous.layout.inline_y;
        if !clean {
            return;
        }
        self.layout.pref_w = previous.layout.pref_w;
        self.layout.min_w = previous.layout.min_w;
        self.layout.pref_h = previous.layout.pref_h;
        self.layout.min_h = previous.layout.min_h;
        self.layout.content_w = previous.layout.content_w;
        self.layout.content_h = previous.layout.content_h;
        self.layout.lines.clone_from(&previous.layout.lines);
        self.layout
            .wrap_lines
            .clone_from(&previous.layout.wrap_lines);
        self.layout.clearance = previous.layout.clearance;
        self.layout.float_h = previous.layout.float_h;
        self.layout.beside = previous.layout.beside;
        self.layout
            .inline_rows
            .clone_from(&previous.layout.inline_rows);
        self.layout
            .rich_lines
            .clone_from(&previous.layout.rich_lines);
        self.layout.rows.clone_from(&previous.layout.rows);
        self.layout.grid.clone_from(&previous.layout.grid);
        self.layout.fit_step = previous.layout.fit_step;
        self.layout.margin_sets = previous.layout.margin_sets;
        self.layout.collapse_in = previous.layout.collapse_in;
        self.layout.through = previous.layout.through;
        self.layout.margin_x = previous.layout.margin_x;
    }

    /// Its content size along `axis` (0 across) under size containment:
    /// its `contain-intrinsic-*` length, its last remembered size for
    /// `auto` while it skips its contents, else 0 (CSS Containment 2
    /// §3.1, CSS Sizing 4 §5). `None` without size containment there.
    pub(crate) fn contained_size(&self, axis: usize) -> Option<f32> {
        let c = &self.layout.used.contain;
        let bit = [
            crate::layout::Contain::SIZE_W,
            crate::layout::Contain::SIZE_H,
        ][axis];
        if !self.layout.cv_skip && !c.has(bit) {
            return None;
        }
        let (auto, len) = c.intrinsic[axis];
        Some(match self.layout.remembered {
            Some(r) if auto && self.layout.cv_skip => r[axis],
            _ => len.unwrap_or(0.0),
        })
    }

    /// How far a column break moved line `k` of its text ([`Node::frag`]).
    pub(crate) fn frag_at(&self, k: usize) -> [f32; 2] {
        self.layout
            .frag
            .iter()
            .rev()
            .find(|f| f.0 as usize <= k)
            .map_or([0.0; 2], |f| f.1)
    }

    /// The scroll range: how far the content overflows the inner height.
    pub fn max_scroll(&self) -> f32 {
        if !self.layout.used.scroll {
            return 0.0;
        }
        (self.layout.content_h - (self.layout.h - self.layout.used.padding.vertical())).max(0.0)
    }

    /// The horizontal scroll range: how far the content overflows the inner width.
    pub fn max_scroll_x(&self) -> f32 {
        if !self.layout.used.scroll_x {
            return 0.0;
        }
        (self.layout.content_w - (self.layout.w - self.layout.used.padding.horizontal())).max(0.0)
    }

    /// Scrolls either way.
    pub fn scrolls(&self) -> bool {
        self.layout.used.scroll || self.layout.used.scroll_x
    }
}
