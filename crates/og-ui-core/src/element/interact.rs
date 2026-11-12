use crate::layout::Rect;
use crate::motion::Transition;

/// Which way a drag moves its element.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragAxis {
    #[default]
    Both,
    X,
    Y,
    /// Either, locked to the first one the drag moves along.
    Lock,
}

/// Swipe to dismiss: a release past `distance` px (or faster than
/// `velocity` px/s) along the drag's axis throws the element off.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dismiss {
    /// 0: 40% of the element's size along the axis.
    pub distance: f32,
    /// 0: 800 px/s.
    pub velocity: f32,
}

/// The element moves with a drag that starts on it (a card, a sheet, a
/// carousel), released onto a spring that keeps the drag's velocity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DragSpec {
    pub axis: DragAxis,
    /// Bounds of the offset (`x..x+width`, `y..y+height`); `None`: free.
    pub bounds: Option<Rect>,
    /// Past the bounds the element follows with resistance, iOS's rubber
    /// band with this constant (0: stops at the bounds; 0.55 is iOS's).
    pub rubber: f32,
    /// Offsets a release settles on: the nearest to where the momentum
    /// would carry it.
    pub snap: Vec<(f32, f32)>,
    /// A release projects the offset by its velocity before choosing a
    /// snap point (a flick).
    pub momentum: bool,
    /// A release goes back to offset 0 when no snap point is given.
    pub return_home: bool,
    pub dismiss: Option<Dismiss>,
    /// Pointer travel before a press becomes a drag, CSS px (0: 4).
    pub threshold: f32,
    /// The release spring (`None`: the style's `ui` spring).
    pub spring: Option<Transition>,
    /// What a drop target matches in [`DropSpec::accept`].
    pub tag: String,
}

/// Two touches on the element scale (and turn) it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PinchSpec {
    /// 0: 0.5.
    pub min_scale: f32,
    /// 0: 4.
    pub max_scale: f32,
    pub rotate: bool,
}

/// The element takes drops of dragged elements.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DropSpec {
    /// Tags it takes (empty: any).
    pub accept: Vec<String>,
}

/// Where keys and pads move the focus from this element (docs/ui.md §2.1),
/// over the spatial pick: each a path (a full one, `Hud/Bag`, or a
/// sibling's id), empty to leave that move to the layout. A path naming a
/// focus group lands on its remembered member (or its first); one naming
/// any other container, on its first focusable control.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NavLinks {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    /// Tab and Shift+Tab.
    pub next: String,
    pub prev: String,
}

impl NavLinks {
    pub fn is_empty(&self) -> bool {
        [
            &self.up,
            &self.down,
            &self.left,
            &self.right,
            &self.next,
            &self.prev,
        ]
        .iter()
        .all(|s| s.is_empty())
    }
}

/// A container whose focusable controls are one group for keys and pads
/// (docs/ui.md §2.1): arrows move among its members first. At its edge a
/// group that wraps goes round to its far side; one that does not leaves
/// it for the nearest control outside that way, and stays put when there
/// is none. The zero value neither wraps nor remembers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusGroup {
    /// At its edge the focus wraps round inside it.
    pub wrap: bool,
    /// Focus coming into it from outside lands on the member it last had.
    pub memory: bool,
}

/// Tap, double tap and long press on any element (docs/ui.md §2.1),
/// recognised from the press, its travel and the ticks it is held, in the
/// UI state. Nested elements' recognisers compete for one press: of those
/// that declare a gesture, the highest `priority` wins it (ties: the
/// innermost). A tap waits for a double tap only where one is declared; a
/// long press that fires takes the press, so its release clicks nothing;
/// travel past `slop` (a drag, a scroll) fails all three. The zero value
/// recognises nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GestureSpec {
    pub tap: bool,
    pub double_tap: bool,
    pub long_press: bool,
    /// How long a long press is held, ms (0: 500).
    pub long_press_ms: f32,
    /// The longest gap between a double tap's two taps, ms (0: 300).
    pub double_tap_ms: f32,
    /// Pointer travel that fails a tap and a long press, CSS px (0: 10).
    pub slop: f32,
    pub priority: i32,
}

impl GestureSpec {
    pub fn any(&self) -> bool {
        self.tap || self.double_tap || self.long_press
    }
}

/// The pointer's shape over an element (CSS `cursor`, CSS UI 4 §5.1,
/// docs/ui.md §2), inherited by its subtree: every keyword CSS has. `Auto`
/// takes its parent's, and with none set above it a text control's I-beam,
/// a draggable element's open hand (a closed one while dragged), a link's
/// hand, else the arrow. `url()` images over a keyword are
/// [`CursorImage`]s beside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Cursor {
    #[default]
    Auto,
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    Move,
    NotAllowed,
    Crosshair,
    Help,
    Wait,
    Progress,
    ResizeEw,
    ResizeNs,
    ResizeNesw,
    ResizeNwse,
    ZoomIn,
    ZoomOut,
    /// Hidden (a game drawing its own).
    None,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    AllScroll,
    ResizeCol,
    ResizeRow,
    ResizeN,
    ResizeE,
    ResizeS,
    ResizeW,
    ResizeNe,
    ResizeNw,
    ResizeSe,
    ResizeSw,
}

/// Each cursor's (variant, IDL name, CSS keyword).
const CURSOR_NAMES: [(Cursor, &str, &str); 36] = [
    (Cursor::Auto, "auto", "auto"),
    (Cursor::Default, "default", "default"),
    (Cursor::Pointer, "pointer", "pointer"),
    (Cursor::Text, "text", "text"),
    (Cursor::Grab, "grab", "grab"),
    (Cursor::Grabbing, "grabbing", "grabbing"),
    (Cursor::Move, "move", "move"),
    (Cursor::NotAllowed, "notAllowed", "not-allowed"),
    (Cursor::Crosshair, "crosshair", "crosshair"),
    (Cursor::Help, "help", "help"),
    (Cursor::Wait, "wait", "wait"),
    (Cursor::Progress, "progress", "progress"),
    (Cursor::ResizeEw, "resizeEw", "ew-resize"),
    (Cursor::ResizeNs, "resizeNs", "ns-resize"),
    (Cursor::ResizeNesw, "resizeNesw", "nesw-resize"),
    (Cursor::ResizeNwse, "resizeNwse", "nwse-resize"),
    (Cursor::ZoomIn, "zoomIn", "zoom-in"),
    (Cursor::ZoomOut, "zoomOut", "zoom-out"),
    (Cursor::None, "none", "none"),
    (Cursor::ContextMenu, "contextMenu", "context-menu"),
    (Cursor::Cell, "cell", "cell"),
    (Cursor::VerticalText, "verticalText", "vertical-text"),
    (Cursor::Alias, "alias", "alias"),
    (Cursor::Copy, "copy", "copy"),
    (Cursor::NoDrop, "noDrop", "no-drop"),
    (Cursor::AllScroll, "allScroll", "all-scroll"),
    (Cursor::ResizeCol, "resizeCol", "col-resize"),
    (Cursor::ResizeRow, "resizeRow", "row-resize"),
    (Cursor::ResizeN, "resizeN", "n-resize"),
    (Cursor::ResizeE, "resizeE", "e-resize"),
    (Cursor::ResizeS, "resizeS", "s-resize"),
    (Cursor::ResizeW, "resizeW", "w-resize"),
    (Cursor::ResizeNe, "resizeNe", "ne-resize"),
    (Cursor::ResizeNw, "resizeNw", "nw-resize"),
    (Cursor::ResizeSe, "resizeSe", "se-resize"),
    (Cursor::ResizeSw, "resizeSw", "sw-resize"),
];

impl Cursor {
    pub const ALL: [Cursor; 36] = {
        let mut all = [Cursor::Auto; 36];
        let mut k = 0;
        while k < all.len() {
            all[k] = CURSOR_NAMES[k].0;
            k += 1;
        }
        all
    };

    /// Its CSS keyword (`auto` for `Auto`).
    pub fn css(self) -> &'static str {
        CURSOR_NAMES[self as usize].2
    }

    /// Its IDL name (`notAllowed`, `resizeEw`…).
    pub fn name(self) -> &'static str {
        CURSOR_NAMES[self as usize].1
    }

    pub fn from_name(name: &str) -> Option<Cursor> {
        CURSOR_NAMES.iter().find(|c| c.1 == name).map(|c| c.0)
    }

    /// The cursor a CSS keyword names.
    pub fn from_css(word: &str) -> Option<Cursor> {
        CURSOR_NAMES
            .iter()
            .find(|c| c.2.eq_ignore_ascii_case(word))
            .map(|c| c.0)
    }
}

/// A `cursor: url(<image>) [<x> <y>]` image tried before the keyword
/// (CSS UI 4 §5.1): an atlas icon by name, or a texture by its asset
/// handle, as `background-image` names one, and its hotspot in image px
/// (`None`: the image's own, else its top left).
#[derive(Clone, Debug, PartialEq)]
pub struct CursorImage {
    pub url: String,
    pub hotspot: Option<[f32; 2]>,
}

/// CSS `pointer-events` (CSS UI 4 §5.3, SVG 2 §16.6), inherited.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointerEvents {
    /// Not declared: its parent's (`Auto` at the top).
    #[default]
    Inherit,
    Auto,
    /// The pointer passes through it to what is under it; its subtree's
    /// elements that say `auto` still take it. Keys and pads still reach it.
    None,
    /// An SVG value: which of an SVG shape's parts take the pointer. On
    /// other elements it is `Auto`.
    Svg(SvgPointer),
}

/// The SVG `pointer-events` values: bit 0 its fill, bit 1 its stroke,
/// bit 2 only while it paints them (`visiblePainted`, `painted`), bit 3
/// only while visible, bit 4 its bounding box.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SvgPointer(pub u8);

impl SvgPointer {
    pub const FILL: u8 = 1;
    pub const STROKE: u8 = 2;
    pub const PAINTED: u8 = 4;
    pub const VISIBLE: u8 = 8;
    pub const BOX: u8 = 16;
}

/// CSS `visibility` (CSS Display 3 §7), inherited: a hidden element keeps
/// its place but draws nothing and takes no pointer or focus; a child that
/// says `visible` inside it is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Not declared: its parent's.
    #[default]
    Inherit,
    Visible,
    Hidden,
    /// `hidden`, except that a flex item gives up its main size (CSS
    /// Flexbox 1 §4.4) and a table row or column its track.
    Collapse,
}

/// An element's interaction properties (CSS UI 4, CSS Containment 2,
/// CSS Will Change 1, Pointer Events 3). The zero value is CSS's initial.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Interact {
    pub pointer_events: PointerEvents,
    pub visibility: Visibility,
    /// CSS `touch-action` (Pointer Events 3 §9): `None` is `auto`, else
    /// the gestures a touch starting on it may make, bits of
    /// [`Interact::PAN_X`], [`Interact::PAN_Y`], [`Interact::PINCH`] (0:
    /// `none`). The used value intersects its ancestors'.
    pub touch_action: Option<u8>,
}

impl Interact {
    pub const PAN_LEFT: u8 = 1;
    pub const PAN_RIGHT: u8 = 2;
    pub const PAN_UP: u8 = 4;
    pub const PAN_DOWN: u8 = 8;
    pub const PAN_X: u8 = Self::PAN_LEFT | Self::PAN_RIGHT;
    pub const PAN_Y: u8 = Self::PAN_UP | Self::PAN_DOWN;
    pub const PINCH: u8 = 16;
}

/// Whether the player can select an element's text and copy it (CSS
/// `user-select`, docs/ui.md §2). `Auto` takes its parent's, and with
/// none set above it (or under a control: a button's label is pressed, not
/// selected) `None`: a game's text is not selectable unless it says so.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UserSelect {
    #[default]
    Auto,
    /// Not selectable.
    None,
    /// A press and drag selects characters, a double press a word, a
    /// triple press a line; a long press on a touch screen a word.
    Text,
    /// A press selects the element's whole text at once.
    All,
}

impl UserSelect {
    pub const ALL: [UserSelect; 4] = [
        UserSelect::Auto,
        UserSelect::None,
        UserSelect::Text,
        UserSelect::All,
    ];

    /// Its IDL and CSS name.
    pub fn name(self) -> &'static str {
        match self {
            UserSelect::Auto => "auto",
            UserSelect::None => "none",
            UserSelect::Text => "text",
            UserSelect::All => "all",
        }
    }

    pub fn from_name(name: &str) -> Option<UserSelect> {
        UserSelect::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// An element the player can move, scale and fade in a HUD layout editor
/// (docs/ui.md §2.1): a HUD group, a touch stick or button. Its placement
/// is kept per player under `key` ([`crate::Ui::hud_layout`]); the zero
/// value allows all three, from half to twice its size and down to a fifth
/// of its opacity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Customize {
    /// Its name in the saved layout (empty: its path).
    pub key: String,
    pub lock_move: bool,
    pub lock_scale: bool,
    pub lock_opacity: bool,
    /// 0: 0.5.
    pub min_scale: f32,
    /// 0: 2.
    pub max_scale: f32,
    /// 0: 0.2.
    pub min_opacity: f32,
}

impl Customize {
    pub(crate) fn scale_range(&self) -> (f32, f32) {
        let lo = if self.min_scale > 0.0 {
            self.min_scale
        } else {
            0.5
        };
        let hi = if self.max_scale > 0.0 {
            self.max_scale
        } else {
            2.0
        };
        (lo, hi.max(lo))
    }

    pub(crate) fn opacity_floor(&self) -> f32 {
        if self.min_opacity > 0.0 {
            self.min_opacity.min(1.0)
        } else {
            0.2
        }
    }
}
