//! Timelines and choreography (docs/ui.md §7.4): keyframed tracks over
//! element props with an ease or a spring per key, event tracks (a sound, a
//! haptic, a burst, a state change, a callback), and the operators that
//! compose timelines into one play: `sequence`, `parallel`, `stagger` (by
//! index, from the end, from the centre or from a point, within a cap) and
//! `cascade` (each step starts at an event of the one before). A play is
//! flattened into clips (a timeline on an element at an offset) on the tick
//! it starts, so only numbers are in flight and in the snapshot.

use super::spring::{Ease, SpringSpec};
use crate::element::Prop;
use crate::style::Srgba;

/// How the segment into a key moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyEase {
    Linear,
    Ease(Ease),
    /// On a spring from the previous key's value (released at it).
    Spring(SpringSpec),
    /// Holds the previous value, then jumps at the key.
    Hold,
}

impl Default for KeyEase {
    fn default() -> KeyEase {
        KeyEase::Ease(Ease::Out)
    }
}

/// A keyframe: a value at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Key {
    pub at_ms: f32,
    pub value: f32,
    pub ease: KeyEase,
}

/// Keys over one prop of one element: the element the clip plays on, or a
/// descendant by its path below it (`Stars/Star#2`).
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
    pub target: String,
    pub prop: Prop,
    /// In time order.
    pub keys: Vec<Key>,
}

/// What an event does when the play reaches it.
#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    /// A UI sound of the style (`hover`, `press`, `open`, `close`,
    /// `success`, `error`) at a pitch (1: as recorded); the event also
    /// reaches the game ([`UiEvent`]) with the cascade step's pitch.
    Sound { name: String, pitch: f32 },
    /// A haptic pattern by name (`light`, `medium`, `heavy`, `success`…),
    /// for the host.
    Haptic(String),
    /// Rays and sparks from the target (`crate::BurstDesc`'s look).
    Burst {
        count: u32,
        radius: f32,
        color: Option<Srgba>,
    },
    /// Sets a state group of the target.
    State { group: String, state: String },
    /// A callback to game code by name ([`UiEvent`]).
    Callback(String),
}

/// An event at a time, on the clip's element or a descendant.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEvent {
    pub at_ms: f32,
    pub target: String,
    pub kind: EventKind,
}

/// Keyframed tracks and events. Before its first key a track holds that
/// key's value, after its last the last.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Timeline {
    pub tracks: Vec<TimelineTrack>,
    pub events: Vec<TimelineEvent>,
    /// Named times (`land`, `count`), for seeking (`PlayDesc::from_label`).
    pub labels: Vec<(String, f32)>,
    /// Its length, ms (0: its last key or event).
    pub duration_ms: f32,
}

impl Timeline {
    pub fn length_ms(&self) -> f32 {
        let keys = self
            .tracks
            .iter()
            .filter_map(|t| t.keys.last().map(|k| k.at_ms))
            .fold(0.0, f32::max);
        let events = self.events.iter().map(|e| e.at_ms).fold(0.0, f32::max);
        if self.duration_ms > 0.0 {
            self.duration_ms
        } else {
            keys.max(events)
        }
    }

    pub fn label(&self, name: &str) -> Option<f32> {
        self.labels.iter().find(|(n, _)| n == name).map(|(_, t)| *t)
    }
}

/// A track's value at `t` ms. Spring segments are stepped tick by tick
/// from their start (arithmetic only), so a seek lands where a play would.
pub(crate) fn sample(track: &TimelineTrack, t: f32, tps: u32) -> Option<f32> {
    let keys = &track.keys;
    let first = keys.first()?;
    if t <= first.at_ms {
        return Some(first.value);
    }
    // Each segment starts where the one before ended (a spring may not
    // have reached its key by the next one).
    let mut start = first.value;
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        let span = (b.at_ms - a.at_ms).max(0.0);
        let local = t - a.at_ms;
        let end_value = |x: f32| segment(start, b, x, span, tps);
        if t <= b.at_ms {
            return Some(end_value(local));
        }
        start = end_value(span);
    }
    Some(start)
}

fn segment(from: f32, key: Key, x: f32, span: f32, tps: u32) -> f32 {
    let k = if span > 0.0 {
        (x / span).clamp(0.0, 1.0)
    } else {
        1.0
    };
    match key.ease {
        KeyEase::Linear => from + (key.value - from) * k,
        KeyEase::Ease(e) => from + (key.value - from) * e.at(k),
        KeyEase::Hold => {
            if k >= 1.0 {
                key.value
            } else {
                from
            }
        }
        KeyEase::Spring(s) => {
            let spring = s.to_tween();
            let dt = 1.0 / tps.max(1) as f32;
            let ticks = (x.max(0.0) * tps as f32 / 1000.0).round() as u32;
            let mut st = og_tween::SpringState {
                value: from,
                velocity: 0.0,
            };
            for _ in 0..ticks.min(10_000) {
                st = spring.step(st, key.value, dt);
            }
            st.value
        }
    }
}

/// Where a stagger's delays grow from.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum StaggerFrom {
    /// The first target first (by index).
    #[default]
    Start,
    /// The last first.
    End,
    /// The targets nearest the centre of their bounds first (by distance).
    Center,
    /// The nearest to a point (CSS px on the surface) first.
    Point(f32, f32),
}

/// Timelines composed into one play.
#[derive(Clone, Debug, PartialEq)]
pub enum Choreo {
    /// A timeline (by name) on an element: the play's element, or a
    /// descendant by its path below it.
    Play {
        timeline: String,
        target: String,
        delay_ms: f32,
    },
    /// One after another.
    Sequence(Vec<Choreo>),
    /// All at once.
    Parallel(Vec<Choreo>),
    /// One timeline on each target (paths below the play's element; empty:
    /// its children in order), each `step_ms` after the one before as
    /// ordered by `from`, the whole spread within `cap_ms` (0: no cap).
    Stagger {
        timeline: String,
        targets: Vec<String>,
        step_ms: f32,
        cap_ms: f32,
        from: StaggerFrom,
    },
    /// Steps where each starts at the first `on` callback event of the one
    /// before (or its end): a score that counts chips, then multiplies,
    /// each step pitched higher ([`UiEvent::index`]).
    Cascade { steps: Vec<Choreo>, on: String },
}

/// How a play repeats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Repeat {
    #[default]
    Once,
    Loop,
    /// Forward then back, over and over.
    PingPong,
}

/// How a play runs. The zero value plays once forward at speed 1 from the
/// start and lets go of its values when it ends.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayDesc {
    pub reverse: bool,
    pub repeat: Repeat,
    /// 0: 1.
    pub speed: f32,
    /// Start this far in, ms.
    pub from_ms: f32,
    /// Start at a label of the first timeline (wins over `from_ms`).
    pub from_label: String,
    /// Keep the last values once it ends (until stopped).
    pub hold: bool,
    /// Decorative: under reduced motion it does not play at all.
    pub decorative: bool,
}

/// What happened in the UI's motion this frame, for the game and the host
/// ([`crate::UiFrame::events`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiEvent {
    /// `sound`, `haptic`, `burst`, `state`, `callback`, `end` (a play
    /// finished), `dismiss` (a swipe threw an element off), `drop` (a drag
    /// was dropped on a target), `snap` (a released drag chose a snap point).
    pub kind: String,
    /// The sound, haptic, callback or state's name; for `drop` the target's
    /// path; for `snap` the point's index.
    pub name: String,
    /// The element's path.
    pub target: String,
    /// The play (0: none).
    pub play: u32,
    /// The clip's step in a cascade or stagger.
    pub index: u32,
    /// A sound's pitch: its own times 1.06 per cascade step.
    pub pitch: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(keys: Vec<Key>) -> TimelineTrack {
        TimelineTrack {
            target: String::new(),
            prop: Prop::Opacity,
            keys,
        }
    }

    #[test]
    fn a_track_holds_before_and_after_and_eases_between() {
        let t = track(vec![
            Key {
                at_ms: 100.0,
                value: 0.0,
                ease: KeyEase::Linear,
            },
            Key {
                at_ms: 300.0,
                value: 1.0,
                ease: KeyEase::Linear,
            },
            Key {
                at_ms: 400.0,
                value: 5.0,
                ease: KeyEase::Hold,
            },
        ]);
        assert_eq!(sample(&t, 0.0, 60), Some(0.0));
        assert_eq!(sample(&t, 200.0, 60), Some(0.5));
        assert_eq!(sample(&t, 350.0, 60), Some(1.0));
        assert_eq!(sample(&t, 401.0, 60), Some(5.0));
    }

    #[test]
    fn a_spring_key_overshoots_and_lands() {
        let t = track(vec![
            Key {
                at_ms: 0.0,
                value: 0.0,
                ease: KeyEase::Linear,
            },
            Key {
                at_ms: 1000.0,
                value: 1.0,
                ease: KeyEase::Spring(SpringSpec::new(400.0, 0.4)),
            },
        ]);
        let peak = (0..60)
            .map(|k| sample(&t, k as f32 * 1000.0 / 60.0, 60).unwrap())
            .fold(0.0, f32::max);
        assert!(peak > 1.05, "the bounce overshoots: {peak}");
        assert!((sample(&t, 1000.0, 60).unwrap() - 1.0).abs() < 0.01);
    }
}
