//! Motion by the tick: the token easings evaluated with arithmetic only, and
//! the transition kinds. Nothing here reads a clock; progress is counted in
//! ticks by the [`crate::Ui`] that owns it.

use crate::theme::{EASE_EXIT, EASE_OUT};

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
