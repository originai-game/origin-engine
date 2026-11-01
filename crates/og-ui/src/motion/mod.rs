//! Motion by the tick (docs/ui.md §7): springs by perceptual duration and
//! bounce that keep their velocity when retargeted ([`spring`]), visual
//! state groups, timelines with keyframes and event tracks and the
//! choreography that composes them ([`timeline`]), presence (enter and
//! exit), layout animation (FLIP), shared elements, gestures, and the
//! style's motion tokens with their reduced-motion fallbacks. Nothing here
//! reads a clock; progress is counted in ticks by the [`crate::Ui`] that
//! owns it, and everything in flight is in [`crate::Ui::save_state`].

pub(crate) mod codec;
pub(crate) mod gesture;
pub(crate) mod run;
pub mod spring;
pub(crate) mod state;
pub mod timeline;

pub use gesture::GestureInfo;
pub use run::{MotionDump, PlayInfo};
pub use spring::{Ease, MotionToken, MotionTokens, Reduced, SpringSpec, Transition};
pub use timeline::{
    Choreo, EventKind, Key, KeyEase, PlayDesc, Repeat, StaggerFrom, Timeline, TimelineEvent,
    TimelineTrack, UiEvent,
};

use crate::element::{Prop, Values};
use crate::theme::{EASE_EXIT, EASE_OUT};

/// How an element arrives or leaves (docs/ui.md §7.5): the values it
/// enters from or exits to, and how it moves between them and its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PresenceSpec {
    pub values: Values,
    /// `Token(Enter)` for an entrance, `Token(Exit)` for an exit are the
    /// usual choices (the zero value is the `ui` spring).
    pub transition: Transition,
    /// Wait this long before moving, ms (a hand-built stagger).
    pub delay_ms: f32,
}

impl PresenceSpec {
    /// One of the transition kinds as a presence: what the kind's
    /// entrance starts from (and its exit goes to), `distance` px for slides.
    pub fn from_kind(kind: TransitionKind, distance: f32, transition: Transition) -> PresenceSpec {
        let mut values = vec![(Prop::Opacity, 0.0)];
        match kind {
            TransitionKind::Fade => {}
            TransitionKind::SlideUp => values.push((Prop::Y, distance)),
            TransitionKind::SlideDown => values.push((Prop::Y, -distance)),
            TransitionKind::SlideLeft => values.push((Prop::X, distance)),
            TransitionKind::SlideRight => values.push((Prop::X, -distance)),
            TransitionKind::Scale => values.push((Prop::Scale, 0.92)),
            TransitionKind::Pop => values.push((Prop::Scale, 0.6)),
        }
        PresenceSpec {
            values,
            transition,
            delay_ms: 0.0,
        }
    }
}

/// One state of a group: the values it sets, and timelines it starts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisualState {
    pub name: String,
    pub values: Values,
    /// A timeline (by name, `Ui::define_timeline`) played on the element
    /// when the state is entered (empty: none).
    pub on_enter: String,
    /// One played when it is left.
    pub on_exit: String,
}

/// The transition between two states of a group (`*` matches any state).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub transition: Transition,
}

/// A visual state group (docs/ui.md §7.3): one of its states is current,
/// and moving between them animates every value that changes. A group
/// named `interaction` is derived from input: `normal`, `hover`,
/// `pressed`, `focused` and `disabled` (a state the group lacks falls back:
/// pressed to hover to normal, focused to hover). Other groups take their
/// state from [`crate::Props::state`], a timeline's state event, or the
/// probe ([`crate::Ui::force_state`]); the first state is the default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateGroup {
    pub name: String,
    pub states: Vec<VisualState>,
    pub transitions: Vec<StateTransition>,
}

impl StateGroup {
    /// The transition from `from` to `to`: the exact pair, then `* -> to`,
    /// `from -> *`, `* -> *`.
    pub fn transition(&self, from: &str, to: &str) -> Option<Transition> {
        let find = |f: &str, t: &str| {
            self.transitions
                .iter()
                .find(|x| x.from == f && x.to == t)
                .map(|x| x.transition)
        };
        find(from, to)
            .or_else(|| find("*", to))
            .or_else(|| find(from, "*"))
            .or_else(|| find("*", "*"))
    }
}

/// A cubic Bézier easing `(x1, y1, x2, y2)` at `x` (0–1): x is solved by
/// bisection (24 steps), so the result depends only on `+ − × ÷` and comparisons.
pub fn cubic_bezier(c: [f32; 4], x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let [x1, y1, x2, y2] = c;
    let at = |p1: f32, p2: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t
    };
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if at(x1, x2, mid) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at(y1, y2, (lo + hi) / 2.0)
}

/// `motion.easing.out`: how things arrive.
pub fn ease_out(x: f32) -> f32 {
    cubic_bezier(EASE_OUT, x)
}

/// `motion.easing.exit`: how things leave.
pub fn ease_exit(x: f32) -> f32 {
    cubic_bezier(EASE_EXIT, x)
}

/// How a transition moves its content in and out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionKind {
    /// Opacity only.
    #[default]
    Fade,
    /// Rises into place from below (and sinks back).
    SlideUp,
    /// Drops into place from above.
    SlideDown,
    /// Enters from the right, moving left.
    SlideLeft,
    /// Enters from the left, moving right.
    SlideRight,
    /// Grows from slightly smaller around its centre.
    Scale,
    /// Springs up from smaller around its centre, overshoots and settles.
    Pop,
}

/// What a transition does to its subtree this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fx {
    pub alpha: f32,
    pub dx: f32,
    pub dy: f32,
    pub scale: f32,
    /// Takes no input (moving, or leaving).
    pub inert: bool,
}

impl Default for Fx {
    fn default() -> Fx {
        Fx {
            alpha: 1.0,
            dx: 0.0,
            dy: 0.0,
            scale: 1.0,
            inert: false,
        }
    }
}

/// The effect of `kind` at eased presence `e` (0 gone, 1 settled), moving
/// `distance` CSS px; `reduced` keeps only the opacity.
pub(crate) fn fx(kind: TransitionKind, e: f32, distance: f32, reduced: bool) -> Fx {
    let away = 1.0 - e;
    let mut f = Fx {
        alpha: e,
        ..Fx::default()
    };
    if reduced {
        return f;
    }
    match kind {
        TransitionKind::Fade => {}
        TransitionKind::SlideUp => f.dy = away * distance,
        TransitionKind::SlideDown => f.dy = -away * distance,
        TransitionKind::SlideLeft => f.dx = away * distance,
        TransitionKind::SlideRight => f.dx = -away * distance,
        TransitionKind::Scale => f.scale = 1.0 - 0.08 * away,
        TransitionKind::Pop => {
            // Arithmetic only: a parabola past 1 that lands back on it.
            f.scale = 0.8 + 0.2 * e + 0.8 * e * e * away;
            f.alpha = (2.0 * e).min(1.0);
        }
    }
    f
}
