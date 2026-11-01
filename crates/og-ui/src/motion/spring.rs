//! Springs defined by perceptual duration and bounce (docs/ui.md §7.2), the
//! eases, transitions and motion tokens, and one animated channel: a value
//! with its velocity that keeps both when its target moves.

use super::cubic_bezier;
use crate::theme::{EASE_EXIT, EASE_OUT};

/// A spring as a designer gives it: how long it takes to feel done
/// (`duration_ms`, perceptual) and how much it overshoots (`bounce`, −1 to
/// 1; 0 settles without overshoot, 0.3 a lively bounce, below 0 slower
/// than critical). The stiffness is `(2π / duration)²`; the damping ratio
/// `1 − bounce` (bounce ≥ 0) or `1 / (1 + bounce)` (bounce < 0), as SwiftUI's
/// and Motion's `spring(duration, bounce)`. Integrated by og-tween's
/// arithmetic-only spring, so every host steps the same bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSpec {
    pub duration_ms: f32,
    pub bounce: f32,
}

impl Default for SpringSpec {
    fn default() -> SpringSpec {
        SpringSpec {
            duration_ms: 350.0,
            bounce: 0.0,
        }
    }
}

impl SpringSpec {
    pub const fn new(duration_ms: f32, bounce: f32) -> SpringSpec {
        SpringSpec {
            duration_ms,
            bounce,
        }
    }

    /// The damping ratio.
    pub fn damping(&self) -> f32 {
        let b = self.bounce.clamp(-0.95, 1.0);
        if b >= 0.0 { 1.0 - b } else { 1.0 / (1.0 + b) }
    }

    /// og-tween's spring (damping ratio and response).
    pub fn to_tween(&self) -> og_tween::Spring {
        og_tween::Spring {
            damping: self.damping(),
            response_ms: self.duration_ms.max(1.0),
        }
    }

    /// From a style's spring (damping ratio and response, `crate::theme::Spring`).
    pub fn from_theme(s: crate::theme::Spring) -> SpringSpec {
        let z = s.damping.max(0.05);
        SpringSpec {
            duration_ms: s.response_ms,
            // The inverse of `damping`: an overdamped spring bounces below 0.
            bounce: if z <= 1.0 { 1.0 - z } else { 1.0 / z - 1.0 },
        }
    }
}

/// An easing curve, evaluated with arithmetic only.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Ease {
    Linear,
    /// `motion.easing.out`: how things arrive (the default).
    #[default]
    Out,
    /// `motion.easing.exit`: how things leave.
    Exit,
    /// Slow at both ends.
    InOut,
    /// A cubic Bézier `(x1, y1, x2, y2)`.
    Bezier([f32; 4]),
}

impl Ease {
    pub fn at(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Ease::Linear => x,
            Ease::Out => cubic_bezier(EASE_OUT, x),
            Ease::Exit => cubic_bezier(EASE_EXIT, x),
            Ease::InOut => cubic_bezier([0.65, 0.0, 0.35, 1.0], x),
            Ease::Bezier(c) => cubic_bezier(*c, x),
        }
    }
}

/// A named motion of the style ([`MotionTokens`]): components use these,
/// so a style's personality changes every motion at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionToken {
    /// Controls: hover, press, toggles.
    Ui,
    /// Panels, sheets, cards moving in their layout.
    Panel,
    /// Rewards, reveals, big moments.
    Emphasis,
    /// Bouncy, casual flourishes.
    Playful,
    /// The style's entrance ease and duration.
    Enter,
    /// The style's exit ease and duration.
    Exit,
}

impl MotionToken {
    pub fn name(self) -> &'static str {
        match self {
            MotionToken::Ui => "ui",
            MotionToken::Panel => "panel",
            MotionToken::Emphasis => "emphasis",
            MotionToken::Playful => "playful",
            MotionToken::Enter => "enter",
            MotionToken::Exit => "exit",
        }
    }

    pub fn from_name(n: &str) -> Option<MotionToken> {
        [
            MotionToken::Ui,
            MotionToken::Panel,
            MotionToken::Emphasis,
            MotionToken::Playful,
            MotionToken::Enter,
            MotionToken::Exit,
        ]
        .into_iter()
        .find(|t| t.name() == n)
    }
}

/// How a value moves to a new target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transition {
    /// At once.
    Instant,
    /// On a spring: retargeting keeps its value and velocity.
    Spring(SpringSpec),
    /// Over a duration on an ease (a retarget starts a new ease from where
    /// it is).
    Tween { duration_ms: f32, ease: Ease },
    /// The style's token.
    Token(MotionToken),
}

impl Default for Transition {
    fn default() -> Transition {
        Transition::Token(MotionToken::Ui)
    }
}

impl Transition {
    /// The transition with the style's tokens resolved.
    pub fn resolve(&self, tokens: &MotionTokens) -> Transition {
        match *self {
            Transition::Token(t) => tokens.get(t),
            other => other,
        }
    }

    /// Its duration in ms (perceptual for a spring).
    pub fn duration_ms(&self, tokens: &MotionTokens) -> f32 {
        match self.resolve(tokens) {
            Transition::Instant => 0.0,
            Transition::Spring(s) => s.duration_ms,
            Transition::Tween { duration_ms, .. } => duration_ms,
            Transition::Token(_) => 0.0,
        }
    }
}

/// What an animation does under reduced motion (docs/ui.md §7.7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reduced {
    /// Movement and scale jump; opacity and colour fade quickly (the default).
    #[default]
    Fade,
    /// Everything jumps.
    None,
    /// Plays as authored (motion that carries meaning, kept small).
    Keep,
}

/// A style's motion personality (docs/ui.md §7.7, §12.2): named springs,
/// entrance and exit eases and durations, the stagger and its cap, the
/// overshoot it allows, and its reduced-motion fallback.
/// [`MotionTokens::default`] is the engine's crisp default, which a
/// style's motion block replaces field by field (`Theme::motion`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub ui: SpringSpec,
    pub panel: SpringSpec,
    pub emphasis: SpringSpec,
    pub playful: SpringSpec,
    pub enter_ease: Ease,
    pub enter_ms: f32,
    pub exit_ease: Ease,
    pub exit_ms: f32,
    /// Between the entrances of a list's items, ms.
    pub stagger_ms: f32,
    /// The most a stagger spreads over, ms (a long list steps faster).
    pub stagger_cap_ms: f32,
    /// The most bounce any spring of the style may have (springs asking for
    /// more are clamped): 0 is a style that never overshoots.
    pub overshoot: f32,
    pub reduced: Reduced,
    /// The fade a `Reduced::Fade` fallback takes, ms.
    pub reduced_ms: f32,
}

impl Default for MotionTokens {
    fn default() -> MotionTokens {
        MotionTokens {
            ui: SpringSpec::new(300.0, 0.0),
            panel: SpringSpec::new(420.0, 0.1),
            emphasis: SpringSpec::new(520.0, 0.3),
            playful: SpringSpec::new(600.0, 0.45),
            enter_ease: Ease::Out,
            enter_ms: crate::theme::DURATIONS_MS.base,
            exit_ease: Ease::Exit,
            exit_ms: crate::theme::DURATIONS_MS.fast,
            stagger_ms: 40.0,
            stagger_cap_ms: 400.0,
            overshoot: 0.5,
            reduced: Reduced::Fade,
            reduced_ms: 120.0,
        }
    }
}

impl MotionTokens {
    pub fn get(&self, t: MotionToken) -> Transition {
        let cap = |s: SpringSpec| SpringSpec {
            bounce: s.bounce.min(self.overshoot),
            ..s
        };
        match t {
            MotionToken::Ui => Transition::Spring(cap(self.ui)),
            MotionToken::Panel => Transition::Spring(cap(self.panel)),
            MotionToken::Emphasis => Transition::Spring(cap(self.emphasis)),
            MotionToken::Playful => Transition::Spring(cap(self.playful)),
            MotionToken::Enter => Transition::Tween {
                duration_ms: self.enter_ms,
                ease: self.enter_ease,
            },
            MotionToken::Exit => Transition::Tween {
                duration_ms: self.exit_ms,
                ease: self.exit_ease,
            },
        }
    }
}

/// One animated value (docs/ui.md §7.2): where it is, how fast it moves
/// (units per second), where it is going, and since which tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Channel {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    /// A tween's start value.
    pub from: f32,
    /// The tick of the last retarget.
    pub since: u64,
    /// The last tick it was stepped (a channel steps once a tick).
    pub stepped: u64,
    /// The transition in flight, packed ([`pack`]).
    pub mode: [f32; 6],
    pub kind: u8,
}

/// `Transition` → (kind, params): 0 instant; 1 spring (duration, bounce);
/// 2 tween (duration, ease code, a Bézier's four control points).
pub(crate) fn pack(t: &Transition, tokens: &MotionTokens) -> (u8, [f32; 6]) {
    match t.resolve(tokens) {
        Transition::Instant | Transition::Token(_) => (0, [0.0; 6]),
        Transition::Spring(s) => (1, [s.duration_ms, s.bounce, 0.0, 0.0, 0.0, 0.0]),
        Transition::Tween { duration_ms, ease } => {
            let (e, c) = match ease {
                Ease::Linear => (0.0, [0.0; 4]),
                Ease::Out => (1.0, [0.0; 4]),
                Ease::Exit => (2.0, [0.0; 4]),
                Ease::InOut => (3.0, [0.0; 4]),
                Ease::Bezier(c) => (4.0, c),
            };
            (2, [duration_ms, e, c[0], c[1], c[2], c[3]])
        }
    }
}

fn ease_of(mode: &[f32; 6]) -> Ease {
    match mode[1] as u32 {
        0 => Ease::Linear,
        2 => Ease::Exit,
        3 => Ease::InOut,
        4 => Ease::Bezier([mode[2], mode[3], mode[4], mode[5]]),
        _ => Ease::Out,
    }
}

impl Channel {
    /// A channel at rest at `value`.
    pub fn at(value: f32, now: u64) -> Channel {
        Channel {
            value,
            velocity: 0.0,
            target: value,
            from: value,
            since: now,
            stepped: now,
            mode: [0.0; 6],
            kind: 0,
        }
    }

    /// Moves its target, keeping where it is and how fast it goes.
    pub fn retarget(&mut self, target: f32, kind: u8, mode: [f32; 6], now: u64) {
        self.target = target;
        self.from = self.value;
        self.since = now;
        self.kind = kind;
        self.mode = mode;
    }

    /// One tick (once per tick; a second call in the same tick answers the
    /// same value).
    pub fn step(&mut self, now: u64, tps: u32, stepped_channel: bool) -> f32 {
        if self.stepped == now {
            return self.value;
        }
        self.stepped = now;
        // A delayed start holds where it is.
        if now < self.since {
            return self.value;
        }
        if stepped_channel || self.kind == 0 {
            self.value = self.target;
            self.velocity = 0.0;
            return self.value;
        }
        let dt = 1.0 / tps.max(1) as f32;
        match self.kind {
            1 => {
                let spring = SpringSpec::new(self.mode[0], self.mode[1]).to_tween();
                // The spring's own scale: og-tween snaps to rest within 1e-3
                // of the target, fine for pixels and colours alike.
                let s = spring.step(
                    og_tween::SpringState {
                        value: self.value,
                        velocity: self.velocity,
                    },
                    self.target,
                    dt,
                );
                self.value = s.value;
                self.velocity = s.velocity;
            }
            _ => {
                let ticks = crate::style::Theme::ticks(self.mode[0], tps).max(1) as f32;
                let k = ((now - self.since) as f32 / ticks).min(1.0);
                let before = self.value;
                self.value = if k >= 1.0 {
                    self.target
                } else {
                    self.from + (self.target - self.from) * ease_of(&self.mode).at(k)
                };
                self.velocity = (self.value - before) * tps as f32;
                if k >= 1.0 {
                    self.velocity = 0.0;
                }
            }
        }
        self.value
    }

    /// At rest on its target.
    pub fn settled(&self) -> bool {
        self.value == self.target && self.velocity == 0.0
    }

    /// Done as the eye reads it: settled, or past its perceptual duration
    /// (a spring's tail aside).
    pub fn done(&self, now: u64, tps: u32) -> bool {
        if self.settled() || self.kind == 0 {
            return true;
        }
        let ticks = crate::style::Theme::ticks(self.mode[0], tps) as u64;
        now.saturating_sub(self.since) >= ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(c: &mut Channel, from_tick: u64, ticks: u64) -> Vec<f32> {
        (from_tick..from_tick + ticks)
            .map(|t| c.step(t, 60, false))
            .collect()
    }

    #[test]
    fn a_retargeted_spring_keeps_its_value_and_velocity() {
        let tokens = MotionTokens::default();
        let (kind, mode) = pack(&Transition::Spring(SpringSpec::new(400.0, 0.2)), &tokens);
        let mut c = Channel::at(0.0, 0);
        c.retarget(100.0, kind, mode, 0);
        let path = run(&mut c, 1, 10);
        let (v, vel) = (c.value, c.velocity);
        assert!(v > 0.0 && v < 100.0 && vel > 0.0, "{path:?}");
        // Interrupted: back towards 0. The next tick continues from where it
        // was, moving the same way at first (its momentum), never jumping.
        c.retarget(0.0, kind, mode, 11);
        let next = c.step(12, 60, false);
        assert!(
            next > v,
            "momentum carries it on past {v} before it turns: {next}"
        );
        assert!((next - v).abs() < vel / 60.0 * 1.5, "no jump: {v} → {next}");
        // An ease restarted the same way would jump to a fresh curve: the
        // spring's step is continuous in velocity as well as value.
        let mut d = c;
        let after = run(&mut d, 13, 200);
        assert_eq!(*after.last().unwrap(), 0.0, "it settles on the new target");
    }

    #[test]
    fn a_spring_is_done_at_its_perceptual_duration() {
        let tokens = MotionTokens::default();
        let (kind, mode) = pack(&Transition::Spring(SpringSpec::new(500.0, 0.3)), &tokens);
        let mut c = Channel::at(0.0, 0);
        c.retarget(1.0, kind, mode, 0);
        run(&mut c, 1, 29);
        assert!(!c.done(29, 60));
        run(&mut c, 30, 1);
        assert!(c.done(30, 60), "30 ticks is 500 ms at 60 Hz");
    }

    #[test]
    fn duration_and_bounce_map_onto_damping() {
        assert_eq!(SpringSpec::new(300.0, 0.0).damping(), 1.0);
        assert!((SpringSpec::new(300.0, 0.3).damping() - 0.7).abs() < 1e-6);
        assert!(SpringSpec::new(300.0, -0.5).damping() > 1.9);
        let back = SpringSpec::from_theme(crate::theme::Spring {
            damping: 0.55,
            response_ms: 120.0,
        });
        assert!((back.damping() - 0.55).abs() < 1e-5 && back.duration_ms == 120.0);
        // An overdamped style spring stays overdamped.
        let slow = SpringSpec::from_theme(crate::theme::Spring {
            damping: 2.0,
            response_ms: 300.0,
        });
        assert!((slow.damping() - 2.0).abs() < 1e-5, "{slow:?}");
    }
}
