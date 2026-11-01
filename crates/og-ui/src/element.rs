//! The element model (docs/ui.md §6.1): what every element carries beyond
//! its box ([`crate::Element`]): a transform applied after layout, opacity,
//! `z` inside its layer, absolute or anchored placement, hit slop, drag and
//! drop, the grid and stack walks, per-breakpoint and container variants,
//! the overlap declaration, a shared-element `match` key, and its motion
//! (`crate::motion`). [`Props`] is set with [`crate::Ui::open_with`] or, for
//! any element, [`crate::Ui::set_props`] right after it is built.
//!
//! Every value is plain data whose zero value is a sensible default
//! (`Props::default()` is an element in the flow, unmoved, opaque).

use crate::layout::{Align, Anchor, Cross, Direction, Insets, Rect, Size};
use crate::motion::{PresenceSpec, Reduced, StateGroup, Transition};
use crate::style::Breakpoint;

/// A 2D affine map of CSS px, DOMMatrix's `a`–`f`:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (y down).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Affine {
    fn default() -> Affine {
        Affine::IDENTITY
    }
}

impl Affine {
    pub const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub const fn translate(x: f32, y: f32) -> Affine {
        Affine {
            e: x,
            f: y,
            ..Affine::IDENTITY
        }
    }

    pub const fn scale(sx: f32, sy: f32) -> Affine {
        Affine {
            a: sx,
            d: sy,
            ..Affine::IDENTITY
        }
    }

    /// A clockwise turn on screen by `angle` radians (y down), with the
    /// engine's deterministic sine and cosine.
    pub fn rotate(angle: f32) -> Affine {
        let (s, c) = og_core::math::sin_cos(angle);
        Affine {
            a: c,
            b: s,
            c: -s,
            d: c,
            ..Affine::IDENTITY
        }
    }

    /// CSS `skew(x, y)`, radians.
    pub fn skew(x: f32, y: f32) -> Affine {
        let tan = |v: f32| {
            if v == 0.0 {
                return 0.0;
            }
            let (s, c) = og_core::math::sin_cos(v);
            s / c
        };
        Affine {
            b: tan(y),
            c: tan(x),
            ..Affine::IDENTITY
        }
    }

    /// `self × o`: `o` first, then `self`.
    pub fn mul(&self, o: &Affine) -> Affine {
        Affine {
            a: self.a * o.a + self.c * o.b,
            b: self.b * o.a + self.d * o.b,
            c: self.a * o.c + self.c * o.d,
            d: self.b * o.c + self.d * o.d,
            e: self.a * o.e + self.c * o.f + self.e,
            f: self.b * o.e + self.d * o.f + self.f,
        }
    }

    /// `self` about the point `(px, py)`: `T(p) · self · T(-p)`.
    pub fn about(&self, px: f32, py: f32) -> Affine {
        Affine::translate(px, py)
            .mul(self)
            .mul(&Affine::translate(-px, -py))
    }

    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    pub fn is_identity(&self) -> bool {
        *self == Affine::IDENTITY
    }

    /// Translation and scale only: it maps a rectangle to a rectangle.
    pub fn is_axis_aligned(&self) -> bool {
        self.b == 0.0 && self.c == 0.0
    }

    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// The inverse (`None` for a map that flattens the plane).
    pub fn invert(&self) -> Option<Affine> {
        let det = self.determinant();
        if det.abs() < 1e-12 || !det.is_finite() {
            return None;
        }
        let k = 1.0 / det;
        Some(Affine {
            a: self.d * k,
            b: -self.b * k,
            c: -self.c * k,
            d: self.a * k,
            e: (self.c * self.f - self.d * self.e) * k,
            f: (self.b * self.e - self.a * self.f) * k,
        })
    }

    /// The axis-aligned bounds of `r` mapped.
    pub fn map_rect(&self, r: &Rect) -> Rect {
        if self.is_identity() {
            return *r;
        }
        let pts = [
            self.apply(r.x, r.y),
            self.apply(r.right(), r.y),
            self.apply(r.x, r.bottom()),
            self.apply(r.right(), r.bottom()),
        ];
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (x, y) in pts {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// The map that takes rectangle `from` onto `to` (translate and scale).
    pub fn rect_to_rect(from: &Rect, to: &Rect) -> Affine {
        let sx = if from.width > 0.0 {
            to.width / from.width
        } else {
            1.0
        };
        let sy = if from.height > 0.0 {
            to.height / from.height
        } else {
            1.0
        };
        Affine {
            a: sx,
            d: sy,
            e: to.x - from.x * sx,
            f: to.y - from.y * sy,
            ..Affine::IDENTITY
        }
    }
}

/// A transform applied after layout, about `pivot` (fractions of the
/// element's box; the zero value's pivot is its centre): CSS's
/// `translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy)` with its
/// `transform-origin`. Hit tests, the probe's rectangles and the
/// accessibility tree's bounds follow it; layout does not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// CSS px.
    pub x: f32,
    pub y: f32,
    /// 1: unscaled.
    pub scale_x: f32,
    pub scale_y: f32,
    /// Radians, clockwise on screen.
    pub rotate: f32,
    /// Radians.
    pub skew_x: f32,
    pub skew_y: f32,
    /// Fractions of the box (0.5, 0.5: the centre).
    pub pivot_x: f32,
    pub pivot_y: f32,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        x: 0.0,
        y: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        rotate: 0.0,
        skew_x: 0.0,
        skew_y: 0.0,
        pivot_x: 0.5,
        pivot_y: 0.5,
    };

    /// Moves nothing (the pivot aside).
    pub fn is_identity(&self) -> bool {
        self.x == 0.0
            && self.y == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
            && self.rotate == 0.0
            && self.skew_x == 0.0
            && self.skew_y == 0.0
    }

    /// The map for an element laid out at `rect`.
    pub fn matrix(&self, rect: &Rect) -> Affine {
        if self.is_identity() {
            return Affine::IDENTITY;
        }
        let px = rect.x + self.pivot_x * rect.width;
        let py = rect.y + self.pivot_y * rect.height;
        let mut m = Affine::translate(self.x, self.y);
        if self.rotate != 0.0 {
            m = m.mul(&Affine::rotate(self.rotate));
        }
        if self.skew_x != 0.0 || self.skew_y != 0.0 {
            m = m.mul(&Affine::skew(self.skew_x, self.skew_y));
        }
        m = m.mul(&Affine::scale(self.scale_x, self.scale_y));
        m.about(px, py)
    }

    /// Mirrored for a right-to-left layout: moves and turns the other way.
    pub fn mirrored(&self) -> Transform {
        Transform {
            x: -self.x,
            rotate: -self.rotate,
            skew_x: -self.skew_x,
            skew_y: -self.skew_y,
            pivot_x: 1.0 - self.pivot_x,
            ..*self
        }
    }
}

/// How an element is placed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Position {
    /// In its parent's flow (the default).
    #[default]
    Flow,
    /// Out of the flow, placed by its [`Edges`] inside its parent's box.
    Absolute,
    /// Out of the flow, placed against another element's rectangle
    /// ([`Props::anchor_to`]), its own `anchor_self` point on the target's
    /// `anchor_at` point, moved by the start and top insets.
    Anchored,
}

/// Insets from the parent's edges (`None`: not set). `start` and `end`
/// follow the writing direction: a right-to-left layout mirrors them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub start: Option<f32>,
    pub top: Option<f32>,
    pub end: Option<f32>,
    pub bottom: Option<f32>,
    /// Centred across the parent (the start inset, if any, moves it).
    pub center_x: bool,
    /// Centred down the parent (the top inset, if any, moves it).
    pub center_y: bool,
}

impl Edges {
    pub fn is_set(&self) -> bool {
        self.start.is_some()
            || self.top.is_some()
            || self.end.is_some()
            || self.bottom.is_some()
            || self.center_x
            || self.center_y
    }
}

/// How a container lays out its children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMode {
    /// The row and column walks (the default; [`crate::Element::direction`]).
    #[default]
    Flex,
    /// Children overlaid in the same rectangle (a card's art, frame, cost
    /// gem and name), each placed by its own insets, else by the
    /// container's `align` (across) and `cross` (down).
    Stack,
    /// Tracks and cells ([`Props::grid`]).
    Grid,
}

/// A grid track's size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Track {
    /// CSS px.
    Px(f32),
    /// A share of the space the other tracks leave.
    Fr(f32),
    /// Its largest single-cell item.
    #[default]
    Auto,
    /// Its content between `min` and `max` px.
    MinMax { min: f32, max: f32 },
    /// A fraction (0–1) of the grid's inner size.
    Percent(f32),
}

impl Track {
    /// `n` tracks of `t` (CSS `repeat(n, t)`).
    pub fn repeat(n: usize, t: Track) -> Vec<Track> {
        vec![t; n]
    }
}

/// Which way auto-placed items fill a grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridFlow {
    /// Along rows, adding rows (the default).
    #[default]
    Row,
    /// Down columns, adding columns.
    Column,
}

/// A grid's tracks and areas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridSpec {
    pub columns: Vec<Track>,
    pub rows: Vec<Track>,
    /// Rows (columns, with a column flow) added for items past the declared ones.
    pub auto_track: Track,
    /// `None`: the element's `gap`.
    pub column_gap: Option<f32>,
    pub row_gap: Option<f32>,
    /// Named areas, one string per row of space-separated cell names (CSS
    /// `grid-template-areas`; `.` is an unnamed cell).
    pub areas: Vec<String>,
    pub flow: GridFlow,
    /// Auto-placement back-fills holes left by spanning items.
    pub dense: bool,
}

/// Where an item sits in its parent grid (zero value: auto-placed, one cell).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridPlace {
    /// The first column, 1-based (0: auto).
    pub column: u16,
    /// The first row, 1-based (0: auto).
    pub row: u16,
    /// Columns spanned (0: 1).
    pub column_span: u16,
    pub row_span: u16,
    /// A named area of the grid (wins over the numbers).
    pub area: String,
}

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

/// An animatable value of an element (docs/ui.md §7.1). Colours are sRGB
/// components 0–1 and alpha; rotation and skew radians.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Prop {
    X,
    Y,
    ScaleX,
    ScaleY,
    /// Both scales (a state or a key that sets it sets both).
    Scale,
    Rotate,
    SkewX,
    SkewY,
    Opacity,
    Radius,
    BackgroundR,
    BackgroundG,
    BackgroundB,
    BackgroundA,
    InkR,
    InkG,
    InkB,
    InkA,
    /// Its `z` (stepped, not tweened).
    Z,
    /// 1: takes no input (stepped): a results screen's buttons wait for it.
    Inert,
}

/// The channels a [`Prop`] drives (`Scale` is two).
pub(crate) const CHANNELS: usize = 19;

impl Prop {
    pub const ALL: [Prop; 20] = [
        Prop::X,
        Prop::Y,
        Prop::ScaleX,
        Prop::ScaleY,
        Prop::Scale,
        Prop::Rotate,
        Prop::SkewX,
        Prop::SkewY,
        Prop::Opacity,
        Prop::Radius,
        Prop::BackgroundR,
        Prop::BackgroundG,
        Prop::BackgroundB,
        Prop::BackgroundA,
        Prop::InkR,
        Prop::InkG,
        Prop::InkB,
        Prop::InkA,
        Prop::Z,
        Prop::Inert,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Prop::X => "x",
            Prop::Y => "y",
            Prop::ScaleX => "scaleX",
            Prop::ScaleY => "scaleY",
            Prop::Scale => "scale",
            Prop::Rotate => "rotate",
            Prop::SkewX => "skewX",
            Prop::SkewY => "skewY",
            Prop::Opacity => "opacity",
            Prop::Radius => "radius",
            Prop::BackgroundR => "backgroundR",
            Prop::BackgroundG => "backgroundG",
            Prop::BackgroundB => "backgroundB",
            Prop::BackgroundA => "backgroundA",
            Prop::InkR => "inkR",
            Prop::InkG => "inkG",
            Prop::InkB => "inkB",
            Prop::InkA => "inkA",
            Prop::Z => "z",
            Prop::Inert => "inert",
        }
    }

    pub fn from_name(name: &str) -> Option<Prop> {
        Prop::ALL.into_iter().find(|p| p.name() == name)
    }

    /// The channels it sets (indices into a value row).
    pub(crate) fn channels(self) -> &'static [usize] {
        match self {
            Prop::X => &[0],
            Prop::Y => &[1],
            Prop::ScaleX => &[2],
            Prop::ScaleY => &[3],
            Prop::Scale => &[2, 3],
            Prop::Rotate => &[4],
            Prop::SkewX => &[5],
            Prop::SkewY => &[6],
            Prop::Opacity => &[7],
            Prop::Radius => &[8],
            Prop::BackgroundR => &[9],
            Prop::BackgroundG => &[10],
            Prop::BackgroundB => &[11],
            Prop::BackgroundA => &[12],
            Prop::InkR => &[13],
            Prop::InkG => &[14],
            Prop::InkB => &[15],
            Prop::InkA => &[16],
            Prop::Z => &[17],
            Prop::Inert => &[18],
        }
    }

    /// A stepped channel: it jumps to its target.
    pub(crate) fn stepped(channel: usize) -> bool {
        channel >= 17
    }
}

/// Values for props: a state's look, a presence's start or end.
pub type Values = Vec<(Prop, f32)>;

/// A screen orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// When a [`Variant`] applies. Every condition given must hold.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    /// The viewport's size classes it applies on (empty: any).
    pub breakpoints: Vec<Breakpoint>,
    pub orientation: Option<Orientation>,
    /// The width range, CSS px (0: unbounded): of the viewport, or with
    /// `container` of the element's own laid-out width (a container query:
    /// the same card in a four-column shop and a one-column phone list).
    pub min_width: f32,
    pub max_width: f32,
    pub container: bool,
}

/// What a variant changes: any of the box's layout props, the arrangement,
/// and animatable values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Patch {
    pub direction: Option<Direction>,
    pub width: Option<Size>,
    pub height: Option<Size>,
    pub padding: Option<Insets>,
    pub gap: Option<f32>,
    pub align: Option<Align>,
    pub cross: Option<Cross>,
    pub wrap: Option<bool>,
    pub layout: Option<LayoutMode>,
    pub columns: Option<Vec<Track>>,
    /// Hide it (and its subtree) where the variant applies.
    pub hidden: Option<bool>,
    pub values: Values,
}

/// A prop override under a condition (docs/ui.md §6.2: breakpoint
/// variants and container queries). Later variants win.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variant {
    pub when: Query,
    pub set: Patch,
}

/// Everything an element carries beyond its box. The zero value is an
/// element in the flow, unmoved and opaque, with no motion.
#[derive(Clone, Debug, PartialEq)]
pub struct Props {
    pub transform: Transform,
    /// 0–1 (1: opaque), multiplied down the tree.
    pub opacity: f32,
    /// Drawing (and hit-testing) order among its siblings: higher draws over
    /// lower without leaving the flow.
    pub z: i32,
    pub position: Position,
    pub insets: Edges,
    /// The path of the element an `Anchored` element is placed against.
    pub anchor_to: String,
    /// The target's point it sits on.
    pub anchor_at: Anchor,
    /// Its own point that sits there.
    pub anchor_self: Anchor,
    /// Extra pointer reach around its box, CSS px.
    pub hit_slop: f32,
    pub layout: LayoutMode,
    pub grid: GridSpec,
    /// Its place in a grid parent.
    pub place: GridPlace,
    /// Its container queries read its own width.
    pub container: bool,
    pub variants: Vec<Variant>,
    /// Paths of elements it overlaps on purpose (an ancestor path covers its
    /// subtree): undeclared overlap is reported ([`crate::UiFrame::overlaps`]).
    pub overlaps: Vec<String>,
    /// A shared-element key: the element with the same key on the screen
    /// being left morphs into this one.
    pub match_key: String,
    pub drag: Option<DragSpec>,
    pub pinch: Option<PinchSpec>,
    pub drop: Option<DropSpec>,
    /// How its animatable values move when they change (`None`: at once).
    pub motion: Option<Transition>,
    /// Moves from its last laid-out rectangle to its new one (FLIP).
    pub layout_motion: Option<Transition>,
    pub enter: Option<PresenceSpec>,
    /// Kept, inert and out of hit tests and the accessibility tree, while
    /// it plays after the element leaves the tree.
    pub exit: Option<PresenceSpec>,
    pub states: Vec<StateGroup>,
    /// The current state of non-derived groups (`selection` → `on`).
    pub state: Vec<(String, String)>,
    /// Its reduced-motion fallback (`None`: the style's).
    pub reduced: Option<Reduced>,
    /// It and its subtree take no input (a state or a timeline sets it:
    /// a results screen's buttons wait for the sequence).
    pub inert: bool,
}

impl Default for Props {
    fn default() -> Props {
        Props {
            transform: Transform::IDENTITY,
            opacity: 1.0,
            z: 0,
            position: Position::Flow,
            insets: Edges::default(),
            anchor_to: String::new(),
            anchor_at: Anchor::default(),
            anchor_self: Anchor::default(),
            hit_slop: 0.0,
            layout: LayoutMode::Flex,
            grid: GridSpec::default(),
            place: GridPlace::default(),
            container: false,
            variants: Vec::new(),
            overlaps: Vec::new(),
            match_key: String::new(),
            drag: None,
            pinch: None,
            drop: None,
            motion: None,
            layout_motion: None,
            enter: None,
            exit: None,
            states: Vec::new(),
            state: Vec::new(),
            reduced: None,
            inert: false,
        }
    }
}

impl Props {
    /// Out of its parent's flow.
    pub fn out_of_flow(&self) -> bool {
        self.position != Position::Flow
    }

    /// Takes part in motion: something can animate it.
    pub(crate) fn animated(&self) -> bool {
        self.motion.is_some() || !self.states.is_empty() || self.enter.is_some()
    }
}

/// An overlap between two elements that neither declared
/// ([`crate::UiFrame::overlaps`], docs/ui.md §6.2): `og ui check` reports
/// it with both paths.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlap {
    /// The element placed over the other (absolute, anchored, or in a
    /// floating group), or the later of two siblings in the flow.
    pub path: String,
    pub other: String,
    /// Where they overlap, CSS px.
    pub rect: Rect,
    /// Both are in their parent's flow (a layout bug, not a placement).
    pub in_flow: bool,
}

/// What a frame rebuilt and laid out (docs/ui.md §6.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutStats {
    /// Elements built this tick.
    pub elements: u32,
    /// Elements whose inputs changed since the last tick (or are new).
    pub rebuilt: u32,
    /// Elements laid out this tick; the rest kept their last layout.
    pub laid_out: u32,
    /// Elements whose draw commands were kept from the last tick.
    pub kept_commands: u32,
}

// ── The element model's part of the frame ──────────────────────────────────

use crate::layout::{Content, Float, NONE, Node, apply_patch, preorder, width_in};
use crate::ui::Ui;

/// Whether `path` is `decl` or inside it (an ancestor path covers its subtree).
fn covers(decl: &str, path: &str) -> bool {
    path == decl || (path.starts_with(decl) && path.as_bytes().get(decl.len()) == Some(&b'/'))
}

/// It draws something of its own.
fn painted(n: &Node) -> bool {
    !n.rect.is_empty()
        && (n.el.background.is_some()
            || !matches!(n.content, Content::None)
            || !matches!(
                n.widget,
                crate::layout::Widget::None | crate::layout::Widget::Spacer
            )
            || !n.shapes.is_empty())
}

impl Ui {
    /// Applies each element's breakpoint and orientation variants (the
    /// container ones wait for its width, in layout).
    pub(crate) fn apply_variants(&mut self) {
        let vp = self.viewport;
        let bp = vp.breakpoint();
        let portrait = vp.height >= vp.width;
        for i in 0..self.nodes.len() {
            if self.nodes[i].props.variants.is_empty() {
                continue;
            }
            let vs = self.nodes[i].props.variants.clone();
            for v in vs.iter().filter(|v| !v.when.container) {
                let q = &v.when;
                let ok = (q.breakpoints.is_empty() || q.breakpoints.contains(&bp))
                    && q.orientation
                        .is_none_or(|o| (o == Orientation::Portrait) == portrait)
                    && width_in(vp.width, q);
                if ok {
                    apply_patch(&mut self.nodes[i], &v.set);
                }
            }
        }
    }

    /// Moves node `i` and its subtree by `(dx, dy)` (with the clips its own
    /// scroll containers make).
    pub(crate) fn shift_subtree(&mut self, i: usize, dx: f32, dy: f32) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let sub = preorder(&self.nodes, i);
        let inside: std::collections::BTreeSet<usize> = sub.iter().copied().collect();
        for &j in &sub {
            let n = &mut self.nodes[j];
            n.x += dx;
            n.y += dy;
            n.rect.x += dx;
            n.rect.y += dy;
            if n.clip_src != NONE && inside.contains(&n.clip_src) {
                n.clip.x += dx;
                n.clip.y += dy;
            }
        }
    }

    /// Where node `i`'s first line of text sits (its baseline, CSS px), if
    /// it or its first descendant in the flow has text.
    fn first_baseline(&self, i: usize) -> Option<f32> {
        let n = &self.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let line = n.lines.first()?;
                let m = &self.fonts.for_text(tc.font, &tc.text).metrics;
                let lh = tc.line_height();
                let inner = n.rect.inset(&n.el.padding);
                let block = lh * n.lines.len() as f32;
                let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                Some(m.baseline(&line.text, top, lh, tc.step.size))
            }
            Content::Rich(_) => {
                let l = n.rich_lines.first()?;
                Some(n.rect.y + n.el.padding.top + l.baseline)
            }
            _ => n.children.iter().find_map(|&c| self.first_baseline(c)),
        }
    }

    /// Rows with `Cross::Baseline`: each child moved down so their first
    /// baselines meet (per line of a wrapping row).
    pub(crate) fn align_baselines(&mut self, order: &[usize]) {
        for &i in order {
            let n = &self.nodes[i];
            if n.el.cross != crate::layout::Cross::Baseline
                || n.el.direction != crate::layout::Direction::Row
                || n.props.layout != LayoutMode::Flex
                || n.children.is_empty()
            {
                continue;
            }
            let kids = n.children.clone();
            let lines = if n.el.wrap && !n.rows.is_empty() {
                n.rows.clone()
            } else {
                vec![(0, kids.len())]
            };
            for (s, e) in lines {
                let bs: Vec<Option<f32>> =
                    kids[s..e].iter().map(|&c| self.first_baseline(c)).collect();
                let Some(max) = bs.iter().flatten().copied().reduce(f32::max) else {
                    continue;
                };
                for (&c, b) in kids[s..e].iter().zip(bs) {
                    if let Some(b) = b {
                        self.shift_subtree(c, 0.0, max - b);
                    }
                }
            }
        }
    }

    /// The node a path names: a full path, else one relative to `from`'s parent.
    fn node_at(&self, path: &str, from: usize, order: &[usize]) -> Option<usize> {
        let full = crate::id::key_of_path(path);
        let parent = self.nodes[from].parent;
        let sibling = if parent == NONE {
            0
        } else {
            path.split('/')
                .fold(self.nodes[parent].key, crate::id::child_key)
        };
        order
            .iter()
            .copied()
            .find(|&j| self.nodes[j].key == full)
            .or_else(|| {
                order
                    .iter()
                    .copied()
                    .find(|&j| self.nodes[j].key == sibling)
            })
    }

    /// Anchored elements onto their targets' rectangles.
    pub(crate) fn place_anchored(&mut self, order: &[usize]) {
        for &i in order {
            if self.nodes[i].props.position != Position::Anchored {
                continue;
            }
            let p = self.nodes[i].props.clone();
            let Some(t) = self.node_at(&p.anchor_to, i, order) else {
                let at = self.path_of(i);
                self.warnings.push(format!(
                    "'{at}' is anchored to '{}', which is not on screen: anchorTo takes a full path (Hud/Coins) or a sibling's id",
                    p.anchor_to
                ));
                continue;
            };
            let tr = self.nodes[t].rect;
            let n = &self.nodes[i];
            let (ax, ay) = p.anchor_at.fractions();
            let (ox, oy) = p.anchor_self.fractions();
            let x = tr.x + ax * tr.width - ox * n.rect.width + p.insets.start.unwrap_or(0.0);
            let y = tr.y + ay * tr.height - oy * n.rect.height + p.insets.top.unwrap_or(0.0);
            let (dx, dy) = (x - n.rect.x, y - n.rect.y);
            self.shift_subtree(i, dx, dy);
        }
    }

    /// Overlaps nothing declared (docs/ui.md §6.2): an absolute, anchored
    /// or floating-group element over another element's drawing, and
    /// siblings in the flow over each other.
    pub(crate) fn find_overlaps(&self, order: &[usize]) -> Vec<Overlap> {
        const MAX: usize = 64;
        let mut out: Vec<Overlap> = Vec::new();
        let mut seen: std::collections::BTreeSet<(usize, usize)> = Default::default();
        let area = |r: &Rect| r.width > 0.5 && r.height > 0.5;
        // Each node's declarations, with its ancestors' (a declaration on a
        // placed group covers what is inside it).
        let declared = |a: usize, b_path: &str| -> bool {
            let mut j = a;
            while j != NONE {
                if self.nodes[j]
                    .props
                    .overlaps
                    .iter()
                    .any(|d| covers(d, b_path))
                {
                    return true;
                }
                j = self.nodes[j].parent;
            }
            false
        };
        // What of a node a scroll leaves in view: its rectangle cut by every
        // scrolling ancestor's (an element scrolled out of a list overlaps
        // nothing drawn outside it).
        let shown = |i: usize| -> Rect {
            let mut r = self.nodes[i].rect;
            let mut j = self.nodes[i].parent;
            while j != NONE {
                let n = &self.nodes[j];
                if n.el.scroll || n.el.scroll_x {
                    r = r.intersect(&n.rect);
                }
                j = n.parent;
            }
            r
        };
        let ancestors = |mut j: usize| {
            let mut set = std::collections::BTreeSet::new();
            while j != NONE {
                set.insert(j);
                j = self.nodes[j].parent;
            }
            set
        };
        // Layers meant to be over the rest (dialogs, menus, tips, toasts).
        let over_layers: std::collections::BTreeSet<usize> = self
            .floating
            .iter()
            .copied()
            .filter(|&f| !matches!(self.nodes[f].float, Float::Anchored { .. }))
            .flat_map(|f| preorder(&self.nodes, f))
            .collect();
        let placed: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| {
                i != 0
                    && !over_layers.contains(&i)
                    && (self.nodes[i].props.out_of_flow()
                        || (self.nodes[i].parent == NONE
                            && matches!(self.nodes[i].float, Float::Anchored { .. })))
            })
            .collect();
        let others: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| i != 0 && !over_layers.contains(&i) && painted(&self.nodes[i]))
            .collect();
        for &p in &placed {
            let sub: std::collections::BTreeSet<usize> =
                preorder(&self.nodes, p).into_iter().collect();
            let up = ancestors(p);
            for &a in sub.iter().filter(|&&a| painted(&self.nodes[a])) {
                for &o in &others {
                    if sub.contains(&o) || up.contains(&o) {
                        continue;
                    }
                    let r = shown(a).intersect(&shown(o));
                    if !area(&r) {
                        continue;
                    }
                    let pair = (a.min(o), a.max(o));
                    if !seen.insert(pair) || out.len() >= MAX {
                        continue;
                    }
                    let (pa, po) = (self.path_of(a), self.path_of(o));
                    if declared(a, &po) || declared(o, &pa) {
                        continue;
                    }
                    out.push(Overlap {
                        path: pa,
                        other: po,
                        rect: r,
                        in_flow: false,
                    });
                }
            }
        }
        // Siblings in the flow of a row or column (a stack overlays them on
        // purpose; a grid's cells are its own).
        for &i in order {
            let n = &self.nodes[i];
            if n.props.layout != LayoutMode::Flex
                || n.children.len() < 2
                || over_layers.contains(&i)
            {
                continue;
            }
            for (k, &a) in n.children.iter().enumerate() {
                for &b in &n.children[k + 1..] {
                    let r = shown(a).intersect(&shown(b));
                    if !area(&r) || out.len() >= MAX {
                        continue;
                    }
                    let (pa, pb) = (self.path_of(a), self.path_of(b));
                    if declared(a, &pb) || declared(b, &pa) {
                        continue;
                    }
                    out.push(Overlap {
                        path: pb,
                        other: pa,
                        rect: r,
                        in_flow: true,
                    });
                }
            }
        }
        out
    }
}

impl Ui {
    /// Which subtrees have the same layout inputs as last frame (docs/ui.md
    /// §6.3), and the frame's counts: elements built and rebuilt.
    pub(crate) fn reuse_check(&mut self) {
        let k = &mut self.kept;
        let nodes = &self.nodes;
        let n = nodes.len();
        let sig = (self.theme, self.direction, self.viewport.dpr());
        let fresh = k.epoch != k.prev_epoch || k.sig != Some(sig);
        k.prev_epoch = k.epoch;
        k.sig = Some(sig);
        k.fresh = fresh;
        k.laid_out = 0;
        let by_key: og_core::IdMap<u64, usize> =
            og_core::IdMap::from_iter(k.prev.iter().enumerate().map(|(i, p)| (p.key, i)));
        k.twin = nodes
            .iter()
            .map(|nd| by_key.get(&nd.key).copied().unwrap_or(NONE))
            .collect();
        k.clean = vec![false; n];
        let mut rebuilt = 0u32;
        let keys = |list: &[usize], all: &[Node]| -> Vec<u64> {
            list.iter().map(|&c| all[c].key).collect()
        };
        // Children come after their parents: backwards is bottom-up.
        for i in (0..n).rev() {
            let a = &nodes[i];
            let t = k.twin[i];
            let Some(b) = k.prev.get(t) else {
                rebuilt += 1;
                continue;
            };
            let same_layout = crate::layout::el_layout_eq(&a.el, &b.el)
                && crate::layout::content_layout_eq(&a.content, &b.content)
                && crate::layout::props_layout_eq(&a.props, &b.props)
                && a.float == b.float
                && a.float == Float::None
                && keys(&a.children, nodes) == keys(&b.children, &k.prev)
                && keys(&a.out, nodes) == keys(&b.out, &k.prev);
            let same = same_layout
                && a.el == b.el
                && a.content == b.content
                && a.widget == b.widget
                && a.shapes == b.shapes
                && a.props == b.props;
            if !same {
                rebuilt += 1;
            }
            k.clean[i] =
                !fresh && same_layout && a.children.iter().chain(&a.out).all(|&c| k.clean[c]);
        }
        self.stats = LayoutStats {
            elements: n as u32,
            rebuilt,
            laid_out: 0,
            kept_commands: 0,
        };
    }
}

impl Ui {
    /// Opens a container with the element model's props (docs/ui.md §6.1):
    /// its children follow until [`Ui::close`].
    pub fn open_with(&mut self, id: impl Into<crate::Id>, el: crate::Element, props: Props) {
        self.open(id, el);
        let i = self.parent();
        self.nodes[i].props = props;
    }

    /// Sets the props of the element built just before (any widget, or a
    /// container just closed). Answers false when there is none.
    pub fn set_props(&mut self, props: Props) -> bool {
        match self.props_mut() {
            Some(p) => {
                *p = props;
                true
            }
            None => false,
        }
    }

    /// The props of the element built just before, to change in place.
    pub fn props_mut(&mut self) -> Option<&mut Props> {
        let i = self.last_closed;
        if i == NONE || i >= self.nodes.len() {
            self.warnings.push(
                "props with no element before it: call ui.props() right after the element".into(),
            );
            return None;
        }
        Some(&mut self.nodes[i].props)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rotation_about_the_pivot_keeps_the_pivot_and_inverts() {
        let r = Rect::new(100.0, 50.0, 40.0, 20.0);
        let t = Transform {
            rotate: 0.5,
            scale_x: 2.0,
            x: 10.0,
            ..Transform::IDENTITY
        };
        let m = t.matrix(&r);
        let (cx, cy) = m.apply(120.0, 60.0);
        assert!((cx - 130.0).abs() < 1e-4 && (cy - 60.0).abs() < 1e-4);
        let inv = m.invert().unwrap();
        let (x, y) = m.apply(101.0, 52.0);
        let (bx, by) = inv.apply(x, y);
        assert!((bx - 101.0).abs() < 1e-3 && (by - 52.0).abs() < 1e-3);
    }
}
