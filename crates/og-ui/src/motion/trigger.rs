//! Triggered animations (CSS Animation Triggers 1): an element defines a
//! timeline trigger (`timeline-trigger`), which activates when its
//! timeline enters the trigger's activation range and deactivates when it
//! leaves its active range (§3.1); an animation names it in
//! `animation-trigger` with the action to take on each (§5.1). Names are
//! global, or kept to a subtree by `trigger-scope` (§2.3); the elements
//! defining one name in one scope are one trigger, active while any of
//! them is in its range.
//!
//! A triggered animation's time is its ticks, as an untriggered one's, but
//! it moves only while its playback plays, forwards or backwards, from the
//! tick its trigger's state changed; before its first action it holds at
//! time 0. A trigger's state and each run's playback are state (saved with
//! the runs); the ranges are read from the laid-out geometry each tick.

use super::css::CssAnimation;
use super::scroll::{TimelineTrigger, progress_on};
use crate::layout::NONE;
use std::collections::BTreeMap;

/// An `<animation-action>` (§5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Play,
    PlayOnce,
    PlayForwards,
    PlayBackwards,
    Pause,
    Reset,
    Replay,
}

impl Action {
    pub fn from_css(word: &str) -> Option<Action> {
        Some(match word {
            "none" => Action::None,
            "play" => Action::Play,
            "play-once" => Action::PlayOnce,
            "play-forwards" => Action::PlayForwards,
            "play-backwards" => Action::PlayBackwards,
            "pause" => Action::Pause,
            "reset" => Action::Reset,
            "replay" => Action::Replay,
            _ => return None,
        })
    }
}

/// One trigger an animation responds to: its name and its actions, the
/// entry action and, optionally, the exit action (a timeline trigger
/// takes one or two; any other count does nothing).
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrigger {
    pub name: String,
    pub actions: Vec<Action>,
}

/// One `animation-trigger` item: `none` (no triggers) or
/// `[<dashed-ident> <animation-action>+]+`, where a name given twice
/// keeps only its last place. `None`: invalid.
pub fn parse_item(item: &str) -> Option<Vec<AnimationTrigger>> {
    let words: Vec<&str> = item.split_whitespace().collect();
    if words == ["none"] {
        return Some(Vec::new());
    }
    let mut out: Vec<AnimationTrigger> = Vec::new();
    for w in words {
        if super::scroll::dashed(w) {
            out.retain(|t| t.name != w);
            out.push(AnimationTrigger {
                name: w.to_string(),
                actions: Vec::new(),
            });
        } else {
            out.last_mut()?.actions.push(Action::from_css(w)?);
        }
    }
    (!out.is_empty() && out.iter().all(|t| !t.actions.is_empty())).then_some(out)
}

/// A trigger instance's state (§2.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerState {
    /// It has never activated.
    #[default]
    Pending,
    Active,
    Inactive,
}

/// A triggered animation's playback: Web Animations' play state, as the
/// actions move it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Playback {
    /// It has been played (not in its initial state).
    pub started: bool,
    /// Asked to play: it moves while it is not finished.
    pub playing: bool,
    /// Its playback rate is negative.
    pub backwards: bool,
    /// The state of each of its triggers it last acted on.
    pub seen: Vec<TriggerState>,
}

/// The state an action reads (§5.1's columns).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Initial,
    Playing,
    Paused,
    Finished,
}

impl Playback {
    /// Finished at `ticks`: at its end playing forwards, at its start
    /// playing backwards.
    pub(crate) fn finished(&self, ticks: u64, end: Option<u64>) -> bool {
        self.started
            && if self.backwards {
                ticks == 0
            } else {
                end.is_some_and(|e| ticks >= e)
            }
    }

    /// Moving at `ticks`.
    pub(crate) fn running(&self, ticks: u64, end: Option<u64>) -> bool {
        self.playing && !self.finished(ticks, end)
    }

    /// Web Animations' `play()`: at the end it is heading for, it starts
    /// again from the other end (an endless animation cannot play
    /// backwards from its start). Whether it went back to time 0.
    fn play(&mut self, ticks: &mut u64, end: Option<u64>) -> bool {
        let mut zero = false;
        if self.backwards {
            if *ticks == 0 {
                let Some(e) = end else { return false };
                *ticks = e;
            }
        } else if end.is_some_and(|e| *ticks >= e) {
            *ticks = 0;
            zero = true;
        }
        self.playing = true;
        self.started = true;
        zero
    }

    /// Applies `action` (§5.1's table) at `ticks` of `end`: its extra
    /// effect, then what the state it was in asks. Whether it went back to
    /// time 0 (its events fire again).
    pub(crate) fn apply(&mut self, action: Action, ticks: &mut u64, end: Option<u64>) -> bool {
        let phase = if !self.started {
            Phase::Initial
        } else if self.finished(*ticks, end) {
            Phase::Finished
        } else if self.playing {
            Phase::Playing
        } else {
            Phase::Paused
        };
        match action {
            Action::None => false,
            Action::Play => phase != Phase::Playing && self.play(ticks, end),
            Action::PlayOnce => {
                matches!(phase, Phase::Initial | Phase::Paused) && self.play(ticks, end)
            }
            Action::PlayForwards | Action::PlayBackwards => {
                self.backwards = action == Action::PlayBackwards;
                phase != Phase::Playing && self.play(ticks, end)
            }
            Action::Pause => {
                if phase == Phase::Playing {
                    self.playing = false;
                }
                false
            }
            Action::Reset => {
                *ticks = 0;
                if phase != Phase::Initial {
                    self.playing = false;
                }
                true
            }
            Action::Replay => {
                *ticks = 0;
                if phase != Phase::Playing {
                    self.play(ticks, end);
                }
                true
            }
        }
    }

    /// Its ticks after one more tick from `ticks`: one on in its
    /// direction while it plays (and `animation-play-state` runs), held
    /// at either end; under reduced motion it lands on the end it heads
    /// for at once.
    pub(crate) fn advance(&self, ticks: u64, end: Option<u64>, paused: bool, reduced: bool) -> u64 {
        if !self.playing || paused {
            return ticks;
        }
        match (self.backwards, end) {
            (true, _) if reduced => 0,
            (true, _) => ticks.saturating_sub(1),
            (false, Some(e)) if reduced => e,
            (false, Some(e)) => (ticks + 1).min(e.max(ticks)),
            (false, None) => ticks + 1,
        }
    }
}

/// The tick count at which `a` ends (its delay and every iteration), at
/// `tps` ticks a second; `None` for an endless one.
pub(crate) fn end_ticks(a: &CssAnimation, tps: u32) -> Option<u64> {
    let total = a.delay_ms + a.duration_ms * a.iterations;
    total
        .is_finite()
        .then(|| (total.max(0.0) * tps as f32 / 1000.0).ceil() as u64)
}

/// The scope trigger name `name` is resolved in for node `i`: the render
/// key of its nearest ancestor (or itself) whose `trigger-scope` names it,
/// else 0, the global scope.
pub(crate) fn scope_of(tree: &crate::ui::Tree, i: usize, name: &str) -> u64 {
    let mut at = i;
    while at != NONE {
        let n = &tree.nodes[at];
        if n.props
            .trigger_scope
            .iter()
            .any(|s| s == name || s == "all")
        {
            return n.key;
        }
        at = n.parent;
    }
    0
}

/// Each trigger instance's state this tick, by its scope and name, from
/// `states` (last tick's): the elements defining it are read at their
/// laid-out geometry; it activates when any is in its activation range
/// and deactivates when none is in its active range. An instance no
/// element defines any more is forgotten.
pub(crate) fn step_triggers(
    states: &mut BTreeMap<(u64, String), TriggerState>,
    interaction: &crate::ui::Interaction,
    tree: &crate::ui::Tree,
) {
    let mut inside: BTreeMap<(u64, String), (bool, bool)> = BTreeMap::new();
    for i in 0..tree.nodes.len() {
        if tree.nodes[i].props.timeline_triggers.is_empty() || !tree.index.has(i) {
            continue;
        }
        for t in &tree.nodes[i].props.timeline_triggers {
            let at = |range| {
                progress_on(i, &t.timeline, range, interaction, tree)
                    .is_some_and(|p| (0.0..=1.0).contains(&p))
            };
            let TimelineTrigger {
                activation, active, ..
            } = t;
            let live = [
                active[0].unwrap_or(activation[0]),
                active[1].unwrap_or(activation[1]),
            ];
            let e = inside
                .entry((scope_of(tree, i, &t.name), t.name.clone()))
                .or_default();
            e.0 |= at(*activation);
            e.1 |= at(live);
        }
    }
    states.retain(|k, _| inside.contains_key(k));
    for (k, (entered, live)) in inside {
        let s = states.entry(k).or_default();
        *s = match *s {
            TriggerState::Active if !live => TriggerState::Inactive,
            TriggerState::Pending | TriggerState::Inactive if entered => TriggerState::Active,
            s => s,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §5.1's table, cell by cell, over a 60-tick animation: the state an
    /// action finds it in (initial, playing, paused, finished) and its
    /// time and direction after.
    #[test]
    fn actions_follow_the_specs_table() {
        let end = Some(60);
        let state = |started, playing, backwards| Playback {
            started,
            playing,
            backwards,
            seen: Vec::new(),
        };
        // (state before, ticks before, action) -> (playing, backwards, ticks)
        let run = |mut p: Playback, mut t: u64, a: Action| {
            p.apply(a, &mut t, end);
            (p.started, p.playing, p.backwards, t)
        };
        let initial = state(false, false, false);
        let playing = state(true, true, false);
        let paused = state(true, false, false);
        let done = state(true, true, false);
        // play: plays from initial, paused and finished (from the start).
        assert_eq!(
            run(initial.clone(), 0, Action::Play),
            (true, true, false, 0)
        );
        assert_eq!(
            run(paused.clone(), 30, Action::Play),
            (true, true, false, 30)
        );
        assert_eq!(run(done.clone(), 60, Action::Play), (true, true, false, 0));
        // play-once: not again once finished.
        assert_eq!(
            run(done.clone(), 60, Action::PlayOnce),
            (true, true, false, 60)
        );
        assert_eq!(
            run(paused.clone(), 30, Action::PlayOnce),
            (true, true, false, 30)
        );
        // play-backwards: from initial it starts at the end; playing, it
        // turns where it is.
        assert_eq!(
            run(initial.clone(), 0, Action::PlayBackwards),
            (true, true, true, 60)
        );
        assert_eq!(
            run(playing.clone(), 30, Action::PlayBackwards),
            (true, true, true, 30)
        );
        // play-forwards from finished backwards (at 0): plays from 0.
        let back_done = state(true, true, true);
        assert_eq!(
            run(back_done, 0, Action::PlayForwards),
            (true, true, false, 0)
        );
        // pause only stops a playing one.
        assert_eq!(
            run(playing.clone(), 30, Action::Pause),
            (true, false, false, 30)
        );
        assert_eq!(
            run(initial.clone(), 0, Action::Pause),
            (false, false, false, 0)
        );
        // reset: to 0, and paused unless it never started.
        assert_eq!(
            run(playing.clone(), 30, Action::Reset),
            (true, false, false, 0)
        );
        assert_eq!(
            run(initial.clone(), 0, Action::Reset),
            (false, false, false, 0)
        );
        // replay: to 0 and playing, whatever it was doing.
        assert_eq!(run(paused, 30, Action::Replay), (true, true, false, 0));
        assert_eq!(run(playing, 30, Action::Replay), (true, true, false, 0));
        // A paused one does not advance; a finished one holds at its end.
        assert_eq!(done.advance(60, end, false, false), 60);
        assert_eq!(state(true, false, false).advance(30, end, false, false), 30);
        assert_eq!(state(true, true, true).advance(0, end, false, false), 0);
    }

    #[test]
    fn animation_trigger_items_parse() {
        let t = |name: &str, actions: &[Action]| AnimationTrigger {
            name: name.into(),
            actions: actions.to_vec(),
        };
        assert_eq!(parse_item("none"), Some(Vec::new()));
        assert_eq!(
            parse_item("--a play-forwards play-backwards --b reset --a play"),
            Some(vec![t("--b", &[Action::Reset]), t("--a", &[Action::Play])])
        );
        assert_eq!(parse_item("--a"), None);
        assert_eq!(parse_item("play --a play"), None);
        assert_eq!(parse_item("--a jump"), None);
    }
}
