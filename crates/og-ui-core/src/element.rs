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

mod frame;
pub mod id;
mod interact;
mod placement;
pub mod transform3d;

pub use interact::*;
pub use og_ui_model::{Affine, Transform};
pub use placement::*;

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
pub const CHANNELS: usize = 21;

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
    /// Paths of elements the engine places it over on purpose (a badge on
    /// its item; an ancestor path covers its subtree, `*` everything): any
    /// other overlap is reported ([`crate::UiFrame::overlaps`]). Documents
    /// do not set it.
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
