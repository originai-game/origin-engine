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
}

impl FontRole {
    /// The role a style's text effects are keyed by (`body`, `display`,
    /// `numeric`): a rounded text takes the display role's.
    pub fn effect_index(self) -> usize {
        match self {
            FontRole::Body => 0,
            FontRole::Display | FontRole::Rounded => 1,
            FontRole::Mono => 2,
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
    let dots = ellipsis_of(m);
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
    Slider {
        t: f32,
        active: bool,
    },
    Toggle {
        on: bool,
        /// Where the knob is on the style's `ui` spring (0 off, 1 on).
        slide: f32,
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

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub key: u64,
    pub id: String,
    pub parent: usize,
    pub children: Vec<usize>,
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
    /// Widget data its children read (a list's row height, a grid's cell size).
    pub aux: [f32; 3],
    /// A heading label: the style's flourish frames it.
    pub heading: bool,
    /// What it tells the accessibility tree beyond its kind.
    pub a11y: crate::a11y::A11y,
    /// Its descendants enter one after another (a style's stagger).
    pub stagger: Option<Stagger>,
    /// The element model beyond its box (`crate::element`).
    pub props: crate::element::Props,
    /// Its children out of the flow (absolute, anchored), in build order;
    /// `children` holds the ones in the flow once the frame ends.
    pub out: Vec<usize>,
    /// A grid's resolved tracks and its children's cells.
    pub grid: Option<GridLayout>,
    /// Where it is drawn: its map from layout to screen (its transforms and
    /// its ancestors'), and its box's bounds there.
    pub world: crate::element::Affine,
    pub screen: Rect,
    /// Layout animation's offset and scale this tick (x, y, sx, sy).
    pub flip: [f32; 4],
    /// A drag's offset and a pinch's scale and turn (x, y, scale, radians).
    pub gesture: [f32; 4],
    /// The player's HUD placement (`crate::hud_layout`): offset, scale and
    /// opacity (x, y, scale, opacity).
    pub custom: [f32; 4],
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
    /// The content's width (its children's extent), for horizontal scrolling.
    pub content_w: f32,
    pub lines: Vec<Line>,
    /// A rich text's lines.
    pub rich_lines: Vec<crate::rich::RichLine>,
    /// A rich text's links, by id, in order.
    pub links: Vec<String>,
    /// A wrapping row's lines: ranges of `children`.
    pub rows: Vec<(usize, usize)>,
    pub clip: Rect,
    pub rect: Rect,
}

impl Node {
    pub fn new(key: u64, id: String, parent: usize, el: Element) -> Node {
        Node {
            key,
            id,
            parent,
            children: Vec::new(),
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
            aux: [0.0; 3],
            heading: false,
            a11y: crate::a11y::A11y::default(),
            stagger: None,
            props: crate::element::Props::default(),
            out: Vec::new(),
            grid: None,
            world: crate::element::Affine::IDENTITY,
            screen: Rect::default(),
            flip: [0.0, 0.0, 1.0, 1.0],
            gesture: [0.0, 0.0, 1.0, 0.0],
            custom: [0.0, 0.0, 1.0, 1.0],
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
            rich_lines: Vec::new(),
            links: Vec::new(),
            rows: Vec::new(),
            clip: Rect::default(),
            rect: Rect::default(),
        }
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
