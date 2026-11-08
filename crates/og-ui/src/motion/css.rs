//! CSS transitions and animations on presentation ticks (docs/ui.md §3.2).
//! Transitions pick a timing per animated channel; animations sample their
//! `@keyframes` from the number of ticks they have been running, so a
//! restore lands on the same value and nothing reads a clock. Runs live in
//! the motion state, outside `world_hash`.

use super::spring::{Ease, Transition};
use super::timeline::EventKind;
use crate::element::{CHANNELS, Prop};

/// One `transition` entry: the channels it times, how, and after what delay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssTransition {
    /// Bit `c`: channel `c` ([`Prop::channels`]).
    pub channels: u32,
    pub transition: Transition,
    pub delay_ms: f32,
}

/// The channels of every transitionable property (`all`).
pub const ALL_CHANNELS: u32 = (1 << CHANNELS) - 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

impl Direction {
    /// Whether iteration `k` (0-based) runs from 100% to 0%.
    fn reversed(self, k: u64) -> bool {
        match self {
            Direction::Normal => false,
            Direction::Reverse => true,
            Direction::Alternate => k % 2 == 1,
            Direction::AlternateReverse => k % 2 == 0,
        }
    }
}

/// One `animation` entry, with `name` qualified by its document
/// (`<document>/<keyframes>`).
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimation {
    pub name: String,
    pub duration_ms: f32,
    pub ease: Ease,
    pub delay_ms: f32,
    /// `f32::INFINITY` for `infinite`.
    pub iterations: f32,
    pub direction: Direction,
    pub fill_backwards: bool,
    pub fill_forwards: bool,
    pub paused: bool,
}

impl Default for CssAnimation {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration_ms: 0.0,
            ease: Ease::from_css("ease").expect("CSS ease"),
            delay_ms: 0.0,
            iterations: 1.0,
            direction: Direction::Normal,
            fill_backwards: false,
            fill_forwards: false,
            paused: false,
        }
    }
}

/// A keyframe of one property: its offset (0–1), value, and the timing
/// function from it to the next keyframe (`None`: the animation's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub offset: f32,
    pub value: f32,
    pub ease: Option<Ease>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeTrack {
    pub prop: Prop,
    /// By offset. A missing 0% or 100% keyframe takes the element's value.
    pub keys: Vec<Keyframe>,
}

/// A parsed `@keyframes` rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Keyframes {
    pub tracks: Vec<KeyframeTrack>,
    /// Events at offsets (`-og-sound`), in each iteration.
    pub events: Vec<(f32, EventKind)>,
}

/// A running CSS animation of an element. Its time is the ticks it has run
/// (paused ticks do not count).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CssRun {
    pub element: u64,
    /// Its place in the element's `animation` list.
    pub index: u32,
    pub name: String,
    pub ticks: u64,
    /// The last tick it was stepped.
    pub stepped: u64,
    /// The active time events were fired up to, ms (negative: in its delay).
    pub last_ms: f32,
}

/// Where an animation is at `ticks`: per channel, its value; and the
/// iteration-local times it passed since `last_ms`, for events.
pub(crate) struct Sample {
    pub values: Vec<(usize, f32)>,
    pub active_ms: f32,
}

impl CssAnimation {
    fn active_total(&self) -> f32 {
        if self.iterations.is_infinite() {
            f32::INFINITY
        } else {
            self.duration_ms * self.iterations
        }
    }

    /// The keyframe progress in ms (0..=duration) at active time `t`, or
    /// `None` when the animation has no effect then.
    fn local_ms(&self, active: f32, reduced: bool) -> Option<f32> {
        let d = self.duration_ms;
        let total = self.active_total();
        let at_end = |s: &Self| {
            // The end of the last iteration, as its direction leaves it.
            let iterations = s.iterations;
            let whole = iterations.floor();
            let (k, frac) = if iterations > 0.0 && iterations == whole {
                (whole as u64 - 1, 1.0)
            } else {
                (whole as u64, iterations - whole)
            };
            let t = frac * d;
            Some(if s.direction.reversed(k) { d - t } else { t })
        };
        if reduced {
            // Reduced motion: a finite animation is at its end at once; an
            // endless one (decorative) does not play.
            return (total.is_finite() && self.fill_forwards)
                .then(|| at_end(self))
                .flatten();
        }
        if active < 0.0 {
            return self
                .fill_backwards
                .then(|| if self.direction.reversed(0) { d } else { 0.0 });
        }
        if active >= total || d <= 0.0 {
            return if self.fill_forwards {
                at_end(self)
            } else {
                None
            };
        }
        let k = (active / d).floor();
        let t = active - k * d;
        Some(if self.direction.reversed(k as u64) {
            d - t
        } else {
            t
        })
    }

    /// The channel values of `keyframes` at `ticks` of running time, over
    /// the element's own values `base`.
    pub(crate) fn sample(
        &self,
        keyframes: &Keyframes,
        ticks: u64,
        tps: u32,
        base: &[f32; CHANNELS],
        reduced: bool,
    ) -> Sample {
        let elapsed = ticks as f32 * 1000.0 / tps.max(1) as f32;
        let active_ms = elapsed - self.delay_ms;
        let mut values = Vec::new();
        if let Some(t) = self.local_ms(active_ms, reduced) {
            for track in &keyframes.tracks {
                for &c in track.prop.channels() {
                    values.push((c, self.track_at(track, t, base[c])));
                }
            }
        }
        Sample { values, active_ms }
    }

    fn track_at(&self, track: &KeyframeTrack, t: f32, base: f32) -> f32 {
        let d = self.duration_ms;
        // Keys in ms, with implicit 0% and 100% keyframes at the element's value.
        let mut keys: Vec<(f32, f32, Option<Ease>)> = Vec::with_capacity(track.keys.len() + 2);
        if track.keys.first().is_none_or(|k| k.offset > 0.0) {
            keys.push((0.0, base, None));
        }
        keys.extend(track.keys.iter().map(|k| (k.offset * d, k.value, k.ease)));
        if track.keys.last().is_none_or(|k| k.offset < 1.0) {
            keys.push((d, base, None));
        }
        if t <= keys[0].0 {
            return keys[0].1;
        }
        for w in keys.windows(2) {
            let ((a_ms, a, ease), (b_ms, b, _)) = (w[0], w[1]);
            if t <= b_ms {
                let span = (b_ms - a_ms).max(0.0);
                let k = if span > 0.0 {
                    ((t - a_ms) / span).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                return a + (b - a) * ease.unwrap_or(self.ease).apply(k);
            }
        }
        keys[keys.len() - 1].1
    }

    /// The events of `keyframes` passed in (`from`, `to`] of active time.
    pub(crate) fn passed<'a>(
        &self,
        keyframes: &'a Keyframes,
        from: f32,
        to: f32,
    ) -> Vec<&'a EventKind> {
        let d = self.duration_ms;
        let total = self.active_total();
        let mut out = Vec::new();
        if keyframes.events.is_empty() || to <= from || d <= 0.0 {
            return out;
        }
        let first = (from.max(0.0) / d).floor() as u64;
        let last = (to.min(total) / d).floor() as u64;
        // A long jump (a restore far ahead) fires at most a few iterations.
        for k in first.max(last.saturating_sub(8))..=last {
            for (offset, kind) in &keyframes.events {
                let local = offset * d;
                let at = k as f32 * d
                    + if self.direction.reversed(k) {
                        d - local
                    } else {
                        local
                    };
                if at > from && at <= to && at <= total {
                    out.push(kind);
                }
            }
        }
        out
    }
}

/// The transition timing for channel `c`: the last entry naming it.
pub(crate) fn transition_for(list: &[CssTransition], c: usize) -> Option<&CssTransition> {
    list.iter().rev().find(|t| t.channels >> c & 1 == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opacity(keys: &[(f32, f32)]) -> Keyframes {
        Keyframes {
            tracks: vec![KeyframeTrack {
                prop: Prop::Opacity,
                keys: keys
                    .iter()
                    .map(|&(offset, value)| Keyframe {
                        offset,
                        value,
                        ease: None,
                    })
                    .collect(),
            }],
            events: Vec::new(),
        }
    }

    fn at(a: &CssAnimation, k: &Keyframes, ms: u64, reduced: bool) -> Option<f32> {
        let mut base = [0.0; CHANNELS];
        base[7] = 0.9;
        // 1000 ticks a second: one tick per ms.
        a.sample(k, ms, 1000, &base, reduced)
            .values
            .first()
            .map(|&(_, v)| v)
    }

    #[test]
    fn iterations_direction_delay_and_fill_follow_css() {
        let linear = Ease::from_css("linear").unwrap();
        let keys = opacity(&[(0.0, 0.0), (1.0, 1.0)]);
        let a = CssAnimation {
            duration_ms: 100.0,
            ease: linear,
            delay_ms: 50.0,
            iterations: 2.5,
            direction: Direction::Alternate,
            ..Default::default()
        };
        assert_eq!(
            at(&a, &keys, 10, false),
            None,
            "delay without backwards fill"
        );
        assert_eq!(at(&a, &keys, 75, false), Some(0.25));
        assert_eq!(
            at(&a, &keys, 175, false),
            Some(0.75),
            "second iteration runs back"
        );
        assert_eq!(at(&a, &keys, 275, false), Some(0.25));
        assert_eq!(
            at(&a, &keys, 400, false),
            None,
            "ended without forwards fill"
        );
        let both = CssAnimation {
            fill_backwards: true,
            fill_forwards: true,
            direction: Direction::AlternateReverse,
            ..a.clone()
        };
        assert_eq!(
            at(&both, &keys, 10, false),
            Some(1.0),
            "backwards fill of a reversed start"
        );
        // 2.5 iterations alternate-reverse end halfway through a reversed third.
        assert_eq!(at(&both, &keys, 400, false), Some(0.5));
        assert_eq!(
            at(&both, &keys, 0, true),
            Some(0.5),
            "reduced: at its end at once"
        );
        let endless = CssAnimation {
            iterations: f32::INFINITY,
            fill_forwards: true,
            ..a
        };
        assert_eq!(at(&endless, &keys, 100_050, false), Some(0.0));
        assert_eq!(at(&endless, &keys, 100, true), None, "reduced: loops stop");
    }

    #[test]
    fn missing_ends_take_the_element_value_and_keyframes_carry_their_timing() {
        let linear = Ease::from_css("linear").unwrap();
        let mut keys = opacity(&[(0.5, 0.1)]);
        keys.tracks[0].keys[0].ease = Some(Ease::from_css("step-end").unwrap());
        let a = CssAnimation {
            duration_ms: 200.0,
            ease: linear,
            ..Default::default()
        };
        assert!(
            (at(&a, &keys, 50, false).unwrap() - 0.5).abs() < 1e-6,
            "0.9 → 0.1 linearly"
        );
        // From 50% the keyframe's own step-end holds until 100%.
        assert_eq!(at(&a, &keys, 150, false), Some(0.1));
        assert_eq!(at(&a, &keys, 199, false), Some(0.1));
    }

    #[test]
    fn events_fire_once_per_iteration_in_the_direction_run() {
        let mut keys = opacity(&[(0.0, 0.0), (1.0, 1.0)]);
        keys.events.push((0.25, EventKind::Callback("tick".into())));
        let a = CssAnimation {
            duration_ms: 100.0,
            iterations: 3.0,
            direction: Direction::Alternate,
            ..Default::default()
        };
        let mut fired = Vec::new();
        let mut last = -1e-3;
        for ms in 0..400 {
            let now = ms as f32;
            fired.extend(a.passed(&keys, last, now).into_iter().map(|_| ms));
            last = now;
        }
        // 25 ms, then 75 ms into the reversed second, then 25 ms into the third.
        assert_eq!(fired, [25, 175, 225]);
    }
}

/// Documents driving transitions and animations tick by tick.
#[cfg(test)]
mod document_tests {
    use crate::doc::UiDocument;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use serde_json::{Value, json};

    const X: usize = 0;
    const SCALE: usize = 2;
    const ROTATE: usize = 4;
    const OPACITY: usize = 7;

    fn ui(style: &str, reduced: bool) -> Ui {
        let mut ui = Ui::new(UiDesc::default());
        let mut theme = *ui.theme();
        theme.reduced_motion = reduced;
        ui.set_theme(theme);
        let text = format!(
            r#"<meta name="screen" content="m" data-layer="hud">
<panel id="Card" class:on={{on}} class:paused={{paused}}/>
<style>{style}</style>"#
        );
        ui.define_document(UiDocument::parse("m.svelte", &text).unwrap());
        ui.push_screen("m", json!({"on": false, "paused": false}))
            .unwrap();
        ui
    }

    /// One tick with `vm`; answers the card's channel values.
    fn tick(ui: &mut Ui, vm: &Value) -> [f32; crate::element::CHANNELS] {
        ui.set_screen_vm("m", vm.clone());
        ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
        ui.end();
        let i = (0..ui.nodes.len())
            .find(|&i| ui.path_of(i) == "m/Card")
            .expect("the card");
        crate::layout::read_values(&ui.nodes[i])
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn transitions_time_each_listed_property_with_its_delay_and_curve() {
        // 60 ticks a second: 100 ms is 6 ticks, 50 ms is 3, 200 ms is 12.
        let mut ui = ui(
            "#Card { width: 40px; height: 20px; transition: all 120ms linear, opacity 100ms linear, translate 200ms linear 50ms, rotate 0s; }
             #Card.on { opacity: 0.2; translate: 60px 0; rotate: 1rad; scale: 2; }",
            false,
        );
        let off = json!({"on": false, "paused": false});
        let on = json!({"on": true, "paused": false});
        tick(&mut ui, &off);
        for j in 0..16 {
            let v = tick(&mut ui, &on);
            let k = j as f32;
            assert!(
                close(v[OPACITY], 1.0 - 0.8 * (k / 6.0).min(1.0)),
                "opacity at {j}: {}",
                v[OPACITY]
            );
            let slide = ((k - 3.0) / 12.0).clamp(0.0, 1.0);
            assert!(close(v[X], 60.0 * slide), "x at {j}: {}", v[X]);
            // `rotate 0s` after `all` jumps; scale takes `all`'s 120 ms, 7.2
            // ticks rounded up to whole ticks: 8.
            assert_eq!(v[ROTATE], 1.0, "rotate at {j}");
            assert!(
                close(v[SCALE], 1.0 + (k / 8.0).min(1.0)),
                "scale at {j}: {}",
                v[SCALE]
            );
        }
    }

    #[test]
    fn several_animations_iterate_alternate_fill_pause_and_follow_selectors() {
        let mut ui = ui(
            "#Card { width: 40px; height: 20px; animation: slide 600ms linear 2 alternate, spin 300ms linear 100ms infinite backwards; }
             #Card.paused { animation-play-state: paused, running; }
             #Card.on { animation: slide 600ms linear 2 alternate, spin 300ms linear 100ms infinite backwards, pop 200ms 1 both; }
             @keyframes slide { to { translate: 90px 0; } }
             @keyframes spin { from { rotate: 0.5rad; } to { rotate: 1.5rad; } }
             @keyframes pop { 0% { scale: 1.5; animation-timing-function: linear; } 100% { scale: 0.5; } }",
            false,
        );
        let vm = |on: bool, paused: bool| json!({"on": on, "paused": paused});
        let ms = |ticks: u32| ticks as f32 * 1000.0 / 60.0;
        let slide = |t: f32| {
            if t >= 1200.0 {
                0.0
            } else if t > 600.0 {
                90.0 * (1200.0 - t) / 600.0
            } else {
                90.0 * t / 600.0
            }
        };
        let spin = |t: f32| {
            let active = t - 100.0;
            if active < 0.0 {
                0.5
            } else {
                0.5 + (active % 300.0) / 300.0
            }
        };
        // Ticks 0..30 run; 30..45 pause the slide only (its running time
        // holds at 29 ticks); 45..90 resume 15 ticks behind.
        for tick_no in 0..90u32 {
            let paused = (30..45).contains(&tick_no);
            let slid = match tick_no {
                0..30 => tick_no,
                30..45 => 29,
                _ => tick_no - 15,
            };
            let v = tick(&mut ui, &vm(false, paused));
            assert!(close(v[X], slide(ms(slid))), "x at {tick_no}: {}", v[X]);
            assert!(
                close(v[ROTATE], spin(ms(tick_no))),
                "rotate at {tick_no}: {}",
                v[ROTATE]
            );
            assert_eq!(v[SCALE], 1.0, "no pop before .on");
        }
        // `.on` adds pop (the others keep running); removing it ends pop;
        // adding it again starts it from its first keyframe.
        for round in 0..2 {
            for j in 0..15u32 {
                let v = tick(&mut ui, &vm(true, false));
                let k = (ms(j) / 200.0).min(1.0);
                assert!(
                    close(v[SCALE], 1.5 - k),
                    "pop round {round} at {j}: {}",
                    v[SCALE]
                );
            }
            let v = tick(&mut ui, &vm(false, false));
            assert_eq!(v[SCALE], 1.0);
        }
    }

    #[test]
    fn reduced_motion_ends_finite_animations_stops_loops_and_jumps_transitions() {
        let mut ui = ui(
            "#Card { width: 40px; height: 20px; -og-reduced-motion: none; transition: opacity 400ms linear;
                     animation: slide 600ms linear forwards, spin 300ms linear infinite; }
             #Card.on { opacity: 0.4; }
             @keyframes slide { to { translate: 90px 0; } }
             @keyframes spin { to { rotate: 2rad; } }",
            true,
        );
        let v = tick(&mut ui, &json!({"on": false, "paused": false}));
        assert_eq!((v[X], v[ROTATE]), (90.0, 0.0));
        let v = tick(&mut ui, &json!({"on": true, "paused": false}));
        assert_eq!(v[OPACITY], 0.4);
    }

    #[test]
    fn transitions_and_animations_in_flight_restore_exactly() {
        let mut ui = ui(
            "#Card { width: 40px; height: 20px; transition: opacity 500ms ease-in-out 100ms; animation: spin 700ms ease 3 alternate-reverse both; }
             #Card.on { opacity: 0.1; }
             @keyframes spin { 25% { rotate: 1rad; animation-timing-function: steps(3, jump-both); } to { rotate: -1rad; } }",
            false,
        );
        let on = json!({"on": true, "paused": false});
        for _ in 0..4 {
            tick(&mut ui, &json!({"on": false, "paused": false}));
        }
        for _ in 0..9 {
            tick(&mut ui, &on);
        }
        let saved = ui.save_state();
        let continued: Vec<_> = (0..40).map(|_| tick(&mut ui, &on)).collect();
        assert!(continued.windows(2).any(|w| w[0][OPACITY] != w[1][OPACITY]));
        ui.restore_state(&saved).unwrap();
        for (j, expected) in continued.into_iter().enumerate() {
            assert_eq!(tick(&mut ui, &on), expected, "tick {j} after restore");
        }
    }

    #[test]
    fn conditional_keyframes_follow_the_conditions_and_restart_on_a_switch() {
        // 60 ticks: grow lasts 600 ms, 36 ticks; scale goes 1 → the target.
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "m.svelte",
                r#"<meta name="screen" content="m" data-layer="hud">
<column id="Box"><panel id="Card"/></column>
<style>
#Box { width: 50%; container-type: inline-size; }
#Card { width: 40px; height: 20px; animation: grow 600ms linear both; }
@keyframes grow { to { scale: 2; } }
@media (width < 500px) { @keyframes grow { to { scale: 5; } } }
@container (width > 400px) { @keyframes grow { to { scale: 3; } } }
</style>"#,
            )
            .unwrap(),
        );
        ui.push_screen("m", json!({})).unwrap();
        let mut at = |width: f32| {
            ui.begin(InputFrame::default(), Viewport::new(width, 480.0));
            ui.end();
            let i = (0..ui.nodes.len())
                .find(|&i| ui.path_of(i) == "m/Box/Card")
                .unwrap();
            crate::layout::read_values(&ui.nodes[i])[SCALE]
        };
        // 640: Box 320, no media or container match: the top-level 2.
        for _ in 0..18 {
            at(640.0);
        }
        let half = at(640.0);
        assert!((half - 1.5).abs() < 0.06, "half way to 2: {half}");
        // 400: the media variant (later in source order) wins and starts
        // again from its first keyframe, so it is near 1, not near 3.
        let restarted = at(400.0);
        assert!(restarted < 1.3, "restarted toward 5: {restarted}");
        for _ in 0..40 {
            at(400.0);
        }
        assert!(close(at(400.0), 5.0), "held at 5");
        // 1000: Box 500 > 400; the container variant is last and holds.
        // Its condition settles after the first build, so it switches within
        // a frame of the resize and runs to its end.
        for _ in 0..40 {
            at(1000.0);
        }
        assert!(close(at(1000.0), 3.0), "held at 3");
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }
}
