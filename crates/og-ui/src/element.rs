//! The element model (docs/ui.md §2): what every element carries beyond
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

/// A map of CSS px, DOMMatrix's `a`–`f` with a projective row `g h i`:
/// `x' = (a·x + c·y + e) / w`, `y' = (b·x + d·y + f) / w`,
/// `w = g·x + h·y + i` (y down). It is affine (`g = h = 0`, `i = 1`)
/// except where a CSS 3D transform puts an element on a plane in
/// perspective; the draw then lays that element's layer through it
/// ([`crate::ui`]'s 3D planes) and hit tests map through its inverse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
    pub g: f32,
    pub h: f32,
    pub i: f32,
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
        g: 0.0,
        h: 0.0,
        i: 1.0,
    };

    /// The 2D affine map `[a, b, c, d, e, f]`.
    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Affine {
        Affine {
            a,
            b,
            c,
            d,
            e,
            f,
            ..Affine::IDENTITY
        }
    }

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
        if self.is_affine() && o.is_affine() {
            return Affine {
                a: self.a * o.a + self.c * o.b,
                b: self.b * o.a + self.d * o.b,
                c: self.a * o.c + self.c * o.d,
                d: self.b * o.c + self.d * o.d,
                e: self.a * o.e + self.c * o.f + self.e,
                f: self.b * o.e + self.d * o.f + self.f,
                ..Affine::IDENTITY
            };
        }
        let (l, r) = (self.rows(), o.rows());
        let at = |i: usize, j: usize| (0..3).map(|k| l[i][k] * r[k][j]).sum::<f32>();
        Affine::from_rows([
            [at(0, 0), at(0, 1), at(0, 2)],
            [at(1, 0), at(1, 1), at(1, 2)],
            [at(2, 0), at(2, 1), at(2, 2)],
        ])
    }

    /// Its 3×3 matrix by rows (`[[a, c, e], [b, d, f], [g, h, i]]`).
    pub fn rows(&self) -> [[f32; 3]; 3] {
        [
            [self.a, self.c, self.e],
            [self.b, self.d, self.f],
            [self.g, self.h, self.i],
        ]
    }

    pub fn from_rows(m: [[f32; 3]; 3]) -> Affine {
        Affine {
            a: m[0][0],
            c: m[0][1],
            e: m[0][2],
            b: m[1][0],
            d: m[1][1],
            f: m[1][2],
            g: m[2][0],
            h: m[2][1],
            i: m[2][2],
        }
    }

    /// No projective part: it keeps parallel lines parallel.
    pub fn is_affine(&self) -> bool {
        self.g == 0.0 && self.h == 0.0 && self.i == 1.0
    }

    /// `self` about the point `(px, py)`: `T(p) · self · T(-p)`.
    pub fn about(&self, px: f32, py: f32) -> Affine {
        Affine::translate(px, py)
            .mul(self)
            .mul(&Affine::translate(-px, -py))
    }

    /// Where `(x, y)` lands (behind the eye, where w is not positive, at
    /// infinity).
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        let (px, py) = (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        );
        if self.is_affine() {
            return (px, py);
        }
        let w = self.g * x + self.h * y + self.i;
        if w <= 1e-6 {
            return (px.signum() * f32::MAX, py.signum() * f32::MAX);
        }
        (px / w, py / w)
    }

    pub fn is_identity(&self) -> bool {
        *self == Affine::IDENTITY
    }

    /// Translation and scale only: it maps a rectangle to a rectangle.
    pub fn is_axis_aligned(&self) -> bool {
        self.b == 0.0 && self.c == 0.0 && self.is_affine()
    }

    /// The determinant of its linear part (its 3×3's for a projective map).
    pub fn determinant(&self) -> f32 {
        if self.is_affine() {
            return self.a * self.d - self.b * self.c;
        }
        let m = self.rows();
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// The inverse (`None` for a map that flattens the plane).
    pub fn invert(&self) -> Option<Affine> {
        let det = self.determinant();
        if det.abs() < 1e-12 || !det.is_finite() {
            return None;
        }
        let k = 1.0 / det;
        if self.is_affine() {
            return Some(Affine {
                a: self.d * k,
                b: -self.b * k,
                c: -self.c * k,
                d: self.a * k,
                e: (self.c * self.f - self.d * self.e) * k,
                f: (self.b * self.e - self.a * self.f) * k,
                ..Affine::IDENTITY
            });
        }
        let m = self.rows();
        let cof = |r0: usize, r1: usize, c0: usize, c1: usize| {
            m[r0][c0] * m[r1][c1] - m[r0][c1] * m[r1][c0]
        };
        Some(Affine::from_rows([
            [
                cof(1, 2, 1, 2) * k,
                -cof(0, 2, 1, 2) * k,
                cof(0, 1, 1, 2) * k,
            ],
            [
                -cof(1, 2, 0, 2) * k,
                cof(0, 2, 0, 2) * k,
                -cof(0, 1, 0, 2) * k,
            ],
            [
                cof(1, 2, 0, 1) * k,
                -cof(0, 2, 0, 1) * k,
                cof(0, 1, 0, 1) * k,
            ],
        ]))
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
    /// Out of the flow, placed by its [`Edges`] in the whole surface, not its
    /// parent's box: under the notch, the status bar and the home bar
    /// (a screen's full-bleed art or backdrop inside a safe-area layer).
    Screen,
    /// Out of the flow, placed against another element's rectangle
    /// ([`Props::anchor_to`]), its own `anchor_self` point on the target's
    /// `anchor_at` point, moved by the start and top insets.
    Anchored,
    /// In the flow, then moved by its insets (CSS `position: relative`):
    /// its siblings keep the room it left.
    Relative,
    /// In the flow, held inside its nearest scroller's scrollport by its
    /// insets while its parent's box is in view (CSS `position: sticky`).
    Sticky,
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
    /// Placed at this fraction of the room its parent leaves across it,
    /// rounded to a whole px (0: at its start, 1: at its end; the start
    /// inset, if any, moves it). Layout places it, so it follows a parent
    /// whose width changed this frame: a slider's knob at its value.
    pub across: Option<f32>,
}

impl Edges {
    pub fn is_set(&self) -> bool {
        self.start.is_some()
            || self.top.is_some()
            || self.end.is_some()
            || self.bottom.is_some()
            || self.center_x
            || self.center_y
            || self.across.is_some()
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
    /// A share of the space the other tracks leave (CSS `minmax(auto, <n>fr)`).
    Fr(f32),
    /// Between its items' least and largest size, growing into free room
    /// (CSS `auto`).
    #[default]
    Auto,
    /// Its content between `min` and `max` px.
    MinMax { min: f32, max: f32 },
    /// A fraction (0–1) of the grid's inner size.
    Percent(f32),
    /// CSS `min-content`: its items' least size.
    MinContent,
    /// CSS `max-content`: its items' largest size.
    MaxContent,
    /// CSS `fit-content(<limit>)`: its items' largest size, no larger than
    /// the limit (but at least their least size).
    FitContent(Breadth),
    /// CSS `minmax(<min>, <max>)` of any two breadths (an `fr` only as the max).
    Range { min: Breadth, max: Breadth },
}

/// One side of a CSS track size (CSS Grid §7.2.3).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Breadth {
    Px(f32),
    /// A fraction (0–1) of the grid's inner size.
    Percent(f32),
    /// A flexible share (a max breadth only).
    Fr(f32),
    #[default]
    Auto,
    MinContent,
    MaxContent,
}

impl Track {
    /// `n` tracks of `t` (CSS `repeat(n, t)`).
    pub fn repeat(n: usize, t: Track) -> Vec<Track> {
        vec![t; n]
    }

    /// Its min and max breadth and its `fit-content()` limit (CSS Grid
    /// §7.2.3): `1fr` is `minmax(auto, 1fr)`, a fixed size both sides.
    pub fn breadths(self) -> (Breadth, Breadth, Option<Breadth>) {
        match self {
            Track::Px(v) => (Breadth::Px(v), Breadth::Px(v), None),
            Track::Fr(f) => (Breadth::Auto, Breadth::Fr(f), None),
            Track::Auto => (Breadth::Auto, Breadth::Auto, None),
            Track::MinMax { min, max } => (Breadth::Px(min), Breadth::Px(max.max(min)), None),
            Track::Percent(p) => (Breadth::Percent(p), Breadth::Percent(p), None),
            Track::MinContent => (Breadth::MinContent, Breadth::MinContent, None),
            Track::MaxContent => (Breadth::MaxContent, Breadth::MaxContent, None),
            Track::FitContent(l) => (Breadth::Auto, Breadth::MaxContent, Some(l)),
            Track::Range { min, max } => (min, max, None),
        }
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

/// What a grid axis's template is (CSS `grid-template-columns`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Template {
    /// Its own tracks.
    #[default]
    Tracks,
    /// CSS `subgrid`: the tracks its area spans in its parent grid.
    Subgrid,
    /// CSS `masonry`: items packed into the other axis's tracks, each where
    /// the track is shortest.
    Masonry,
}

/// A track list's `repeat(auto-fill | auto-fit, …)`: as many repetitions
/// as fit the grid's definite size (at least one).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutoRepeat {
    /// The explicit track it goes before.
    pub at: usize,
    /// `auto-fit`: repeated tracks no item takes collapse to nothing.
    pub fit: bool,
    pub tracks: Vec<Track>,
    /// The names of each of its lines (`tracks.len() + 1`).
    pub names: Vec<Vec<String>>,
    /// The names written right after it.
    pub after: Vec<String>,
}

/// A grid axis's line names, auto repetition and kind beside its tracks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lines {
    /// The names of each explicit line, 0 to the track count (empty: none).
    /// With `repeat`, the entry at its `at` holds the names before it.
    pub names: Vec<Vec<String>>,
    pub repeat: Option<AutoRepeat>,
    pub kind: Template,
}

/// A grid's tracks and areas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridSpec {
    pub columns: Vec<Track>,
    pub rows: Vec<Track>,
    /// The tracks added past the declared ones, on either axis, where
    /// `auto_columns`/`auto_rows` name none.
    pub auto_track: Track,
    /// CSS `grid-auto-columns` and `grid-auto-rows`: the implicit tracks'
    /// sizes, repeating.
    pub auto_columns: Vec<Track>,
    pub auto_rows: Vec<Track>,
    /// Line names, auto repetition and subgrid or masonry, per axis
    /// (columns, rows).
    pub lines: [Lines; 2],
    /// `None`: the element's `gap`.
    pub column_gap: Option<f32>,
    pub row_gap: Option<f32>,
    /// Named areas, one string per row of space-separated cell names (CSS
    /// `grid-template-areas`; `.` is an unnamed cell).
    pub areas: Vec<String>,
    pub flow: GridFlow,
    /// Auto-placement back-fills holes left by spanning items.
    pub dense: bool,
    /// CSS `justify-content` and `align-content` of its tracks (`None`:
    /// `normal`, its `auto` tracks stretch into free room).
    pub content: [Option<crate::layout::Align>; 2],
    /// CSS `align-items` (`None`: `normal`, items stretch down their area).
    pub align_items: Option<crate::layout::Cross>,
}

/// One end of a grid item's placement on an axis (CSS Grid §8.3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum GridLine {
    /// Auto-placed (or the numbers of [`GridPlace`]).
    #[default]
    Auto,
    /// The `n`th line (negative: from the end), or the `n`th line called
    /// `name`; `n` 0 with a name: the name alone (an area's `name-start`
    /// or `name-end` line, else the first line called it).
    Line(i32, String),
    /// `span <n> [<name>]`: `n` tracks (or up to the `n`th line called
    /// `name`) from the other end.
    Span(u32, String),
}

impl GridLine {
    /// CSS `<grid-line>`: `auto`, `<integer> [<name>]`, `<name>`, `span
    /// [<integer>] [<name>]`.
    pub fn parse(s: &str) -> Result<GridLine, String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        let ident = |w: &str| {
            w.chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_' || c == '-')
                && !matches!(w, "auto" | "span")
        };
        let int = |w: &str| w.parse::<i32>().ok();
        let how = || {
            format!("'{s}' is not a grid line: auto, <n> [<name>], <name> or span [<n>] [<name>]")
        };
        let (span, rest) = match words.split_first() {
            Some((&"span", rest)) => (true, rest),
            _ => (false, words.as_slice()),
        };
        let (mut n, mut name) = (None, String::new());
        for &w in rest {
            match int(w) {
                Some(v) if n.is_none() => n = Some(v),
                None if ident(w) && name.is_empty() => name = w.to_string(),
                _ => return Err(how()),
            }
        }
        Ok(match (span, n) {
            (false, _) if rest == ["auto"] => GridLine::Auto,
            (false, Some(0)) => return Err("a grid line's number is not 0".into()),
            (false, Some(v)) => GridLine::Line(v, name),
            (false, None) if !name.is_empty() => GridLine::Line(0, name),
            (true, Some(v)) if v > 0 => GridLine::Span(v as u32, name),
            (true, None) if !name.is_empty() => GridLine::Span(1, name),
            _ => return Err(how()),
        })
    }
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
    /// CSS `grid-column-start`/`-end` and `grid-row-start`/`-end`, per axis
    /// (columns, rows) as (start, end); either set wins over the numbers.
    pub lines: [[GridLine; 2]; 2],
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

/// An animatable value of an element (docs/ui.md §3.1). Colours are sRGB
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
    /// Its subtree blurred by this many CSS px (CSS `blur()`'s standard
    /// deviation): Svelte's `blur` transition.
    Blur,
    /// How much of its stroke is hidden from the end (0 drawn whole, 1
    /// none), over the stroke's own length: Svelte's `draw` transition.
    Draw,
}

/// The channels a [`Prop`] drives (`Scale` is two).
pub(crate) const CHANNELS: usize = 21;

impl Prop {
    pub const ALL: [Prop; 22] = [
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
        Prop::Blur,
        Prop::Draw,
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
            Prop::Blur => "blur",
            Prop::Draw => "draw",
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
            Prop::Blur => &[19],
            Prop::Draw => &[20],
        }
    }

    /// A stepped channel: it jumps to its target.
    pub(crate) fn stepped(channel: usize) -> bool {
        matches!(channel, 17 | 18)
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
    /// `container` of the element's own laid-out inline size, its width
    /// (its height in a vertical writing mode: a container query, the same
    /// card in a four-column shop and a one-column phone list).
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

/// A prop override under a condition (docs/ui.md §2: breakpoint
/// variants and container queries). Later variants win.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Variant {
    pub when: Query,
    pub set: Patch,
}

/// An out-of-flow element placed at a point of its parent's view box (an
/// SVG `<text>`): laid out unscaled, in view units as CSS px, then moved so
/// its anchor sits on `at` fitted into the parent's box (`xMidYMid meet`,
/// as the parent's shapes are) and scaled by the fit about that anchor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewPlace {
    /// `min-x min-y width height`.
    pub view_box: [f32; 4],
    /// The point, view units.
    pub at: [f32; 2],
    /// The anchor across its own box: 0 its left, 0.5 its centre, 1 its
    /// right (SVG's `text-anchor` in its direction).
    pub anchor_x: f32,
    /// The anchor down its own box, CSS px from its top (its baseline).
    pub anchor_y: f32,
}

/// Everything an element carries beyond its box. The zero value is an
/// element in the flow, unmoved and opaque, with no motion.
#[derive(Clone, Debug, PartialEq)]
pub struct Props {
    pub transform: Transform,
    /// CSS `transform`, `perspective` and the other 3D properties, over
    /// `transform` (`None`: none of them).
    pub space3d: Option<std::sync::Arc<crate::transform3d::Space3d>>,
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
    /// CSS `anchor-name`: the names (`--name`) it is an anchor by.
    pub anchor_names: Vec<String>,
    /// CSS anchor positioning, and insets as percentages (an absolute or
    /// screen element; `None`: its insets are px).
    pub anchor: Option<Box<crate::layout::anchor::AnchorSpec>>,
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
    /// subtree, `*` everything): undeclared overlap is reported
    /// ([`crate::UiFrame::overlaps`]).
    pub overlaps: Vec<String>,
    /// Its text and its subtree's are a dense HUD's readouts (docs/ui.md
    /// §15.2: strategy, MMO, simulation): `og ui check` holds them to 12 px
    /// at 1080p instead of 18, still growing with the player's text scale
    /// and passing the contrast rule. Menus, body text and controls never
    /// take it.
    pub dense: bool,
    /// A shared-element key: the element with the same key on the screen
    /// being left morphs into this one.
    pub match_key: String,
    /// CSS `view-transition-name` (empty: none): its group in a view
    /// transition (`auto` and `match-element` name it by its key).
    pub view_transition_name: String,
    /// CSS `view-transition-class`: what its group is styled by.
    pub view_transition_class: Vec<String>,
    /// CSS `view-transition-group` (empty: normal): where its group nests.
    pub view_transition_group: String,
    pub drag: Option<DragSpec>,
    pub pinch: Option<PinchSpec>,
    pub drop: Option<DropSpec>,
    /// It listens to pointer events (a document's `onpointerdown`,
    /// `onpointermove`, `onpointerup` or `onwheel`): the pointer hits it,
    /// as it hits any element in HTML.
    pub pointer: bool,
    /// Placed at a point of its parent's view box (an SVG `<text>`): see
    /// [`ViewPlace`].
    pub view_place: Option<ViewPlace>,
    /// Explicit focus moves from it (keys and pads), over the spatial pick.
    pub nav: NavLinks,
    /// Its focusable controls are one focus group.
    pub focus_group: Option<FocusGroup>,
    /// A split screen's region: player `k`'s view (0 to 3). Pad `k`'s focus
    /// moves only among the controls inside it (pad 0's is the keyboard's
    /// and the pointer's), its first move lands at its centre, and what is
    /// pressed inside it says `k` ([`crate::Response::player`]). The
    /// controls outside every region are the pads' without one.
    pub player: Option<u8>,
    /// Tap, double tap and long press on it.
    pub gestures: Option<GestureSpec>,
    /// The pointer's shape over it and its subtree.
    pub cursor: Cursor,
    /// `cursor: url()` images tried in order before `cursor`.
    pub cursor_images: Vec<CursorImage>,
    /// Its CSS interaction properties.
    pub interact: Interact,
    /// Whether its text can be selected and copied (`user-select`).
    pub select: UserSelect,
    /// Its text set along a path (`-og-text-path`); `None`: in lines.
    pub text_path: Option<crate::text_path::TextPath>,
    /// The player can move, scale and fade it (a customisable HUD).
    pub customize: Option<Customize>,
    /// How its animatable values move when they change (`None`: at once).
    pub motion: Option<Transition>,
    /// CSS `transition` entries: per-channel timing and delay, over `motion`.
    pub transitions: Vec<crate::motion::css::CssTransition>,
    /// CSS `animation` entries, in list order (later ones win per channel).
    pub animations: Vec<crate::motion::css::CssAnimation>,
    /// Moves from its last laid-out rectangle to its new one (FLIP).
    pub layout_motion: Option<Transition>,
    pub enter: Option<PresenceSpec>,
    /// Its `@starting-style` values: where its first style's channels
    /// start, moved by its CSS transitions.
    pub starting: Values,
    /// Timelines it declares (`scroll-timeline`, `view-timeline`).
    pub timelines: Vec<crate::motion::scroll::NamedTimeline>,
    /// `timeline-scope`: names its subtree's declarations are seen by
    /// (`all` for every one).
    pub timeline_scope: Vec<String>,
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
    /// A document element's direction and bidi ([`crate::layout::Flow`]).
    pub flow: crate::layout::Flow,
}

impl Default for Props {
    fn default() -> Props {
        Props {
            player: None,
            transform: Transform::IDENTITY,
            space3d: None,
            opacity: 1.0,
            z: 0,
            position: Position::Flow,
            insets: Edges::default(),
            anchor_to: String::new(),
            anchor_at: Anchor::default(),
            anchor_self: Anchor::default(),
            anchor_names: Vec::new(),
            anchor: None,
            hit_slop: 0.0,
            layout: LayoutMode::Flex,
            grid: GridSpec::default(),
            place: GridPlace::default(),
            container: false,
            variants: Vec::new(),
            overlaps: Vec::new(),
            dense: false,
            match_key: String::new(),
            view_transition_name: String::new(),
            view_transition_class: Vec::new(),
            view_transition_group: String::new(),
            drag: None,
            pinch: None,
            drop: None,
            pointer: false,
            view_place: None,
            nav: NavLinks::default(),
            focus_group: None,
            gestures: None,
            cursor: Cursor::Auto,
            cursor_images: Vec::new(),
            interact: Interact::default(),
            select: UserSelect::Auto,
            text_path: None,
            customize: None,
            motion: None,
            transitions: Vec::new(),
            animations: Vec::new(),
            layout_motion: None,
            enter: None,
            starting: Vec::new(),
            timelines: Vec::new(),
            timeline_scope: Vec::new(),
            exit: None,
            states: Vec::new(),
            state: Vec::new(),
            reduced: None,
            inert: false,
            flow: crate::layout::Flow::default(),
        }
    }
}

impl Props {
    /// Out of its parent's flow.
    pub fn out_of_flow(&self) -> bool {
        !matches!(
            self.position,
            Position::Flow | Position::Relative | Position::Sticky
        )
    }

    /// Takes part in motion: something can animate it.
    pub(crate) fn animated(&self) -> bool {
        self.motion.is_some()
            || !self.transitions.is_empty()
            || !self.animations.is_empty()
            || !self.states.is_empty()
            || self.enter.is_some()
    }
}

/// An overlap between two elements that neither declared
/// ([`crate::UiFrame::overlaps`], docs/ui.md §2): `og ui check` reports
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

/// What a frame rebuilt and laid out (docs/ui.md §2).
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
    /// Bindings of mounted documents evaluated this tick.
    pub bindings: u32,
    /// Mounted elements whose render nodes were written this tick; the
    /// rest kept theirs without being visited.
    pub reconciled: u32,
    /// Nodes whose draw commands were made this tick.
    pub painted: u32,
    /// Animated nodes the motion pass visited this tick (written ones and
    /// those still moving); the rest kept their values unvisited.
    pub animated: u32,
}

// ── The element model's part of the frame ──────────────────────────────────

use crate::layout::{Content, Float, NONE, Node, apply_patch, preorder, width_in};
use crate::ui::Ui;

/// Whether `path` is `decl` or inside it (an ancestor path covers its subtree).
fn covers(decl: &str, path: &str) -> bool {
    // `*`: over anything on purpose (a dragged card's targeting arc, a
    // ghost following the pointer across the screen).
    decl == "*"
        || path == decl
        || (path.starts_with(decl) && path.as_bytes().get(decl.len()) == Some(&b'/'))
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
        for k in 0..self.emitted.len() {
            let i = self.emitted[k];
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
    /// scroll containers make). With `layout` its laid-out position moves
    /// too, which a clean subtree keeps into the next frame; without it only
    /// where it draws, so a move made again each frame does not add up.
    pub(crate) fn shift_subtree(&mut self, i: usize, dx: f32, dy: f32, layout: bool) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let sub = preorder(&self.nodes, i);
        let inside: std::collections::BTreeSet<usize> = sub.iter().copied().collect();
        for &j in &sub {
            let n = &mut self.nodes[j];
            if layout {
                n.x += dx;
                n.y += dy;
            }
            n.rect.x += dx;
            n.rect.y += dy;
            n.laid_rect.x += dx;
            n.laid_rect.y += dy;
            if n.clip_src != NONE && inside.contains(&n.clip_src) {
                n.clip.x += dx;
                n.clip.y += dy;
                n.laid_clip.x += dx;
                n.laid_clip.y += dy;
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
                let m = &self
                    .fonts
                    .for_styled(tc.font, tc.step.weight, false, &tc.text)
                    .0
                    .metrics;
                let lh = tc.line_height();
                let inner = n.rect.inset(&n.el.padding);
                let block = lh * n.lines.len() as f32;
                let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                Some(m.baseline(&line.text, top, lh, tc.step.size))
            }
            Content::Rich(_) => {
                let l = n.rich_lines.first()?;
                // A first line floats moved down (`Node::wrap_lines`).
                let drop = n.wrap_lines.first().map_or(0.0, |w| w[1]);
                Some(n.rect.y + n.el.padding.top + drop + l.baseline)
            }
            _ => n.children.iter().find_map(|&c| self.first_baseline(c)),
        }
    }

    /// Where node `i`'s last line of text sits, if it or its last
    /// descendant in the flow with text has one.
    fn last_baseline(&self, i: usize) -> Option<f32> {
        let n = &self.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let line = n.lines.last()?;
                let m = &self
                    .fonts
                    .for_styled(tc.font, tc.step.weight, false, &tc.text)
                    .0
                    .metrics;
                let lh = tc.line_height();
                let inner = n.rect.inset(&n.el.padding);
                let block = lh * n.lines.len() as f32;
                let top = inner.y + ((inner.height - block) / 2.0).max(0.0);
                let top = top + lh * (n.lines.len() - 1) as f32;
                Some(m.baseline(&line.text, top, lh, tc.step.size))
            }
            Content::Rich(_) => {
                let (last, before) = n.rich_lines.split_last()?;
                let above: f32 = before.iter().map(|l| l.height).sum();
                let drop = n.wrap_lines.last().map_or(0.0, |w| w[1]);
                Some(n.rect.y + n.el.padding.top + above + drop + last.baseline)
            }
            _ => n.children.iter().rev().find_map(|&c| self.last_baseline(c)),
        }
    }

    /// Where node `i`'s first (`last`) line of text has its central
    /// baseline across a vertical line: the middle of its first (last)
    /// column, physical x. A vertical line's dominant baseline is central
    /// (CSS Writing Modes 4 §4.2); a horizontal text in it (an orthogonal
    /// flow) has none.
    fn central_baseline(&self, i: usize, last: bool) -> Option<f32> {
        let n = &self.nodes[i];
        match &n.content {
            Content::Text(tc) => {
                let v = tc.vertical?;
                let k = if last {
                    n.lines.len().checked_sub(1)?
                } else {
                    0
                };
                n.lines.first()?;
                let lh = tc.line_height();
                let inner = n.rect.inset(&n.el.padding);
                let off = ((inner.width - lh * n.lines.len() as f32) / 2.0).max(0.0);
                Some(if v.mode.block_rl() {
                    inner.right() - off - (k as f32 + 0.5) * lh
                } else {
                    inner.x + off + (k as f32 + 0.5) * lh
                })
            }
            Content::None if last => n
                .children
                .iter()
                .rev()
                .find_map(|&c| self.central_baseline(c, last)),
            Content::None => n
                .children
                .iter()
                .find_map(|&c| self.central_baseline(c, last)),
            _ => None,
        }
    }

    /// Rows with `Cross::Baseline`: each child moved down so their first
    /// baselines meet (per line of a wrapping row). A vertical element's
    /// rows are its lines down the page: their items move across, from
    /// its block-start side, so their central baselines meet.
    pub(crate) fn align_baselines(&mut self, order: &[usize]) {
        use crate::layout::Cross;
        for &i in order {
            let n = &self.nodes[i];
            if !crate::ui::visit::baseline_row(n)
                || n.el.direction != crate::layout::Direction::Row
                || n.props.layout != LayoutMode::Flex
                || n.children.is_empty()
            {
                continue;
            }
            let parent = n.el.cross;
            // Across a vertical line: +1 from a left block-start, -1 from
            // a right one (`Element::vertical`'s sides).
            let across =
                n.el.vertical
                    .filter(|s| s[0] % 2 == 1)
                    .map(|s| if s[0] == 3 { 1.0 } else { -1.0 });
            let kids = n.children.clone();
            let lines = if n.el.wrap && !n.rows.is_empty() {
                n.rows.clone()
            } else {
                vec![(0, kids.len())]
            };
            // A line's first-baseline items share their first baselines, and
            // its last-baseline ones their last (CSS Box Alignment §9.3).
            // A line whose line box holds a `vertical-align` was placed by
            // layout (`Node::inline_rows`), its baselines with it.
            let held: Vec<bool> = (0..lines.len())
                .map(|r| n.inline_rows.get(r).copied().flatten().is_some())
                .collect();
            for (s, e, last) in lines
                .into_iter()
                .zip(held)
                .filter(|&(_, held)| !held)
                .flat_map(|((s, e), _)| [(s, e, false), (s, e, true)])
            {
                let want = if last {
                    Cross::LastBaseline
                } else {
                    Cross::Baseline
                };
                let bs: Vec<Option<f32>> = kids[s..e]
                    .iter()
                    .map(|&c| {
                        if self.nodes[c].el.align_self.unwrap_or(parent) != want {
                            None
                        } else if let Some(dir) = across {
                            self.central_baseline(c, last).map(|x| dir * x)
                        } else if last {
                            self.last_baseline(c)
                        } else {
                            self.first_baseline(c)
                        }
                    })
                    .collect();
                let Some(max) = bs.iter().flatten().copied().reduce(f32::max) else {
                    continue;
                };
                // Where it draws only: a clean row keeps its children's
                // laid-out places into the next frame, and a shift kept
                // there was made again on top of itself every frame (a
                // slider's label and its value in two faces, 0.38 px
                // apart, sank 14 px a second).
                for (&c, b) in kids[s..e].iter().zip(bs) {
                    if let Some(b) = b {
                        match across {
                            Some(dir) => self.shift_subtree(c, dir * (max - b), 0.0, false),
                            None => self.shift_subtree(c, 0.0, max - b, false),
                        }
                    }
                }
            }
        }
    }

    /// The node a path names: a full path, else one relative to `from`'s parent.
    pub(crate) fn node_at(&self, path: &str, from: usize) -> Option<usize> {
        crate::ui::node_in(&self.tree, &self.nodes, path, from)
    }

    /// Anchored elements onto their targets' rectangles.
    pub(crate) fn place_anchored(&mut self, order: &[usize]) {
        for &i in order {
            if self.nodes[i].props.position != Position::Anchored {
                continue;
            }
            let p = self.nodes[i].props.clone();
            let Some(t) = self.node_at(&p.anchor_to, i) else {
                let at = self.path_of(i);
                self.warnings.push(format!(
                    "'{at}' is anchored to '{}', which is not on screen: anchorTo takes a full path (Hud/Coins) or a sibling's id",
                    p.anchor_to
                ));
                continue;
            };
            // Its target's box before a right-to-left mirror, as laid out
            // this frame or kept from the last.
            let tr = self.nodes[t].laid_rect;
            let n = &self.nodes[i];
            let (ax, ay) = p.anchor_at.fractions();
            let (ox, oy) = p.anchor_self.fractions();
            let x = tr.x + ax * tr.width - ox * n.rect.width + p.insets.start.unwrap_or(0.0);
            let y = tr.y + ay * tr.height - oy * n.rect.height + p.insets.top.unwrap_or(0.0);
            let (dx, dy) = (x - n.rect.x, y - n.rect.y);
            self.shift_subtree(i, dx, dy, true);
        }
    }

    /// Overlaps nothing declared (docs/ui.md §2): an absolute, anchored
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
        // Where a node is drawn by its own and its ancestors' declared
        // transforms (a key cap scaled down to fit its corner covers its
        // scaled box, not the line it was laid out on); motion, which
        // comes later, is not part of the layout checked here.
        let posed = |i: usize| -> Rect {
            let mut m = Affine::IDENTITY;
            let mut j = i;
            while j != NONE {
                let n = &self.nodes[j];
                let transform = if self.direction == og_i18n::Direction::Rtl {
                    n.props.transform.mirrored()
                } else {
                    n.props.transform
                };
                if !transform.is_identity() {
                    m = transform.matrix(&n.rect).mul(&m);
                }
                j = n.parent;
            }
            if m.is_identity() {
                self.nodes[i].rect
            } else {
                m.map_rect(&self.nodes[i].rect)
            }
        };
        // What of a node a scroll leaves in view: its rectangle cut by every
        // scrolling ancestor's (an element scrolled out of a list overlaps
        // nothing drawn outside it), and by every ancestor that clips its
        // children to its shape (a map's view larger than its frame draws
        // only inside the frame).
        let clips = |j: usize| -> bool {
            self.draw_of(j).is_some_and(|d| {
                matches!(
                    d.clip,
                    crate::paint::Clip::Shape | crate::paint::Clip::Cut(_)
                )
            })
        };
        // Each node's is worked out once: the pairs below ask for the same
        // node's again and again (every sibling of a long list against every
        // other, every element of a placed group against every element), and
        // each answer walks its ancestors.
        let shown_at: std::cell::RefCell<Vec<Option<Rect>>> =
            std::cell::RefCell::new(vec![None; self.nodes.len()]);
        let shown = |i: usize| -> Rect {
            if let Some(r) = shown_at.borrow()[i] {
                return r;
            }
            let mut r = posed(i);
            let mut j = self.nodes[i].parent;
            while j != NONE {
                let n = &self.nodes[j];
                if n.el.scroll || n.el.scroll_x || clips(j) {
                    r = r.intersect(&posed(j));
                }
                j = n.parent;
            }
            shown_at.borrow_mut()[i] = Some(r);
            r
        };
        // Drawn at all: an element its own or an ancestor's opacity takes to
        // nothing (a counter's invisible sizer, holding its widest value's
        // room) covers nothing.
        let seen_at: std::cell::RefCell<Vec<Option<bool>>> =
            std::cell::RefCell::new(vec![None; self.nodes.len()]);
        let seen_at_all = |i: usize| -> bool {
            if let Some(s) = seen_at.borrow()[i] {
                return s;
            }
            let mut j = i;
            let mut s = true;
            while j != NONE {
                if self.nodes[j].props.opacity <= 0.0 {
                    s = false;
                    break;
                }
                j = self.nodes[j].parent;
            }
            seen_at.borrow_mut()[i] = Some(s);
            s
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
            .filter(|&i| {
                i != 0 && !over_layers.contains(&i) && painted(&self.nodes[i]) && seen_at_all(i)
            })
            .collect();
        for &p in &placed {
            let sub: std::collections::BTreeSet<usize> =
                preorder(&self.nodes, p).into_iter().collect();
            let up = ancestors(p);
            for &a in sub
                .iter()
                .filter(|&&a| painted(&self.nodes[a]) && seen_at_all(a))
            {
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
            // The pairs whose spans along the flow can meet, by a sweep
            // from the start edge (a long list's rows meet only their
            // neighbours, where a check of every pair is quadratic), then
            // in the children's order, as each pair is reported.
            let column = n.el.direction == Direction::Column;
            let span = |r: &Rect| {
                if column {
                    (r.y, r.y + r.height)
                } else {
                    (r.x, r.x + r.width)
                }
            };
            let mut by_start: Vec<(usize, (f32, f32))> = n
                .children
                .iter()
                .enumerate()
                .filter(|&(_, &c)| seen_at_all(c))
                .map(|(k, &c)| (k, span(&shown(c))))
                .collect();
            by_start.sort_by(|x, y| x.1.0.total_cmp(&y.1.0).then(x.0.cmp(&y.0)));
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            for (s, &(ka, (_, end))) in by_start.iter().enumerate() {
                for &(kb, (start, _)) in &by_start[s + 1..] {
                    if start > end {
                        break;
                    }
                    pairs.push((ka.min(kb), ka.max(kb)));
                }
            }
            pairs.sort_unstable();
            for (ka, kb) in pairs {
                let (a, b) = (n.children[ka], n.children[kb]);
                {
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
    /// Which nodes can reuse layout, stopping propagation at unchanged fixed
    /// boxes, and the frame's counts: elements built and rebuilt.
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
        k.full = fresh || k.whole;
        k.unreach();
        // A retained node not written this frame keeps its own inputs.
        if k.full {
            k.clean = vec![!fresh; n];
            k.unclean.clear();
        } else {
            k.clean.resize(n, true);
            for i in std::mem::take(&mut k.unclean) {
                if let Some(c) = k.clean.get_mut(i) {
                    *c = true;
                }
            }
        }
        let mut written = vec![false; n];
        let mut rebuilt = 0u32;
        for &i in &self.emitted {
            written[i] = true;
        }
        // What may lay out otherwise than last frame: the nodes written
        // again, the parents of written and removed ones, every layer and
        // scroll container, a container query's reader, and the subtrees
        // layout moves after it places them; with their ancestors. Every
        // other node keeps its layout, so the walk leaves it out.
        let roots: Vec<usize> = std::iter::once(0)
            .chain(self.floating.iter().copied())
            .collect();
        if !k.full {
            let mut sources: Vec<usize> = Vec::with_capacity(self.emitted.len() * 2);
            for &i in &self.emitted {
                sources.push(i);
                sources.push(nodes[i].parent);
            }
            sources.extend(std::mem::take(&mut k.rederive));
            sources.extend(roots.iter().copied());
            sources.extend(k.scrollers.iter().copied());
            for &i in &self.emitted {
                let a = &nodes[i];
                let moved = a.props.position == Position::Anchored
                    || a.props.position == Position::Screen
                    || a.props.variants.iter().any(|v| v.when.container)
                    || (a.el.cross == crate::layout::Cross::Baseline
                        && a.el.direction == crate::layout::Direction::Row);
                if moved {
                    k.forced.insert(i);
                } else {
                    k.forced.remove(&i);
                }
            }
            let restored = std::mem::take(&mut k.restored);
            for i in sources {
                if i < n {
                    k.reach(nodes, i, false);
                }
            }
            let forced: Vec<usize> = k.forced.iter().copied().collect();
            for i in forced.into_iter().chain(restored) {
                if i < n && nodes[i].key != 0 {
                    k.reach(nodes, i, true);
                }
            }
        }
        // Subtrees laid out afresh (`LayoutCache::relayout`).
        let mut afresh = vec![false; if k.relayout.is_empty() { 0 } else { n }];
        for r in std::mem::take(&mut k.relayout) {
            if r >= n {
                continue;
            }
            k.reach(nodes, r, true);
            let mut stack = vec![r];
            while let Some(j) = stack.pop() {
                afresh[j] = true;
                stack.extend_from_slice(&nodes[j].ordered_children);
            }
        }
        let mut reflowed = Vec::new();
        // Children before parents: the reverse of a preorder of every tree.
        let mut order = Vec::new();
        for &r in &roots {
            order.extend(k.walk(nodes, r));
        }
        for &i in order.iter().rev() {
            let a = &nodes[i];
            let same_layout = if written[i] {
                let Some(b) = k.prev.get(&i) else {
                    rebuilt += 1;
                    k.clean[i] = false;
                    k.unclean.push(i);
                    continue;
                };
                // Its children's margins collapse through their edges only
                // in a block's flow: a change there is theirs too.
                if a.el.block != b.el.block || a.props.layout != b.props.layout {
                    reflowed.push(i);
                }
                let same_layout = crate::layout::el_layout_eq(&a.el, &b.el)
                    && crate::layout::content_layout_eq(&a.content, &b.content)
                    && crate::layout::props_layout_eq(&a.props, &b.props)
                    && a.float == b.float
                    && a.float == Float::None
                    && a.children == b.children
                    && a.out == b.out
                    && a.floats == b.floats;
                let same = same_layout
                    && a.el == b.el
                    && a.content == b.content
                    && a.widget == b.widget
                    && a.shapes == b.shapes
                    && a.props == b.props;
                if !same {
                    rebuilt += 1;
                }
                same_layout
            } else {
                a.float == Float::None && !a.props.variants.iter().any(|v| v.when.container)
            };
            k.clean[i] = !fresh
                && same_layout
                && !afresh.get(i).copied().unwrap_or(false)
                && a.children.iter().chain(&a.out).chain(&a.floats).all(|&c| {
                    let child = &nodes[c];
                    // A box among floats changes what they shorten and
                    // move; its size alone is not all it gives its parent.
                    if child.beside || !child.floats.is_empty() {
                        return k.clean[c];
                    }
                    // A fixed box still lays out its dirty contents, but
                    // those contents cannot change its parent's sizing.
                    k.clean[c]
                        || matches!(child.el.width, Size::Fixed(_))
                            && matches!(child.el.height, Size::Fixed(_))
                            // Nor can they through its edge, unless a
                            // child's top margin collapses through it.
                            && (written[c] || !child.collapse_in[0])
                            && (!written[c]
                                || k.prev.get(&c).is_some_and(|old| {
                                    child.el.width == old.el.width
                                        && child.el.height == old.el.height
                                        && child.el.margin == old.el.margin
                                        && !old.collapse_in[0]
                                        && child.props.place == old.props.place
                                        && child.props.position == old.props.position
                                        && child.props.insets == old.props.insets
                                        && child.props.view_place == old.props.view_place
                                        && child.float == old.float
                                }))
                });
            if !k.clean[i] {
                k.unclean.push(i);
            }
        }
        for i in reflowed {
            for &c in &nodes[i].children {
                if k.clean[c] {
                    k.clean[c] = false;
                    k.unclean.push(c);
                }
            }
        }
        for &i in &self.emitted {
            if let Some(previous) = k.prev.get(&i) {
                self.nodes[i].inherit_layout(previous, k.clean[i]);
            }
        }
        self.stats = LayoutStats {
            rebuilt,
            bindings: self.docs.retained.evaluated as u32,
            reconciled: self.docs.retained.reconciled as u32,
            ..LayoutStats::default()
        };
    }
}

impl Ui {
    /// Opens a container with the element model's props (docs/ui.md §2):
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
