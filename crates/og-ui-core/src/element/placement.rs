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
