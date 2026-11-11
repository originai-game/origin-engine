//! `og.ui`'s motion (docs/ui.md §3.1): the JSON of `UiTimeline`, `Choreo`
//! and `Transition` read into og-ui's types, and the calls that take them.

use super::{paint, with_ui};
use crate::wire::{self, ToJson};
use og_ui::{
    Choreo, Ease, EventKind, Key, KeyEase, MotionToken, PlayDesc, Prop, Repeat, SpringSpec,
    StaggerFrom, Theme, Timeline, TimelineEvent, TimelineTrack, Transition,
};
use serde_json::{Value, json};

fn spring(s: &wire::SpringSpec) -> SpringSpec {
    SpringSpec::new(s.duration_ms, s.bounce)
}

/// An ease: four bezier controls when given, else a named ease (empty:
/// ease-out); `None` for an invalid one.
fn ease(name: &str, bezier: &[f32]) -> Option<Ease> {
    if !bezier.is_empty() {
        let &[a, b, c, d] = bezier else {
            return None;
        };
        let curve = Ease::CubicBezier([a, b, c, d]);
        return curve.is_valid().then_some(curve);
    }
    if name.is_empty() {
        Some(Ease::CubicBezier(og_ui::theme::EASE_OUT))
    } else {
        Ease::from_text(name)
    }
}

/// A `Transition` union; `None` for an invalid ease.
pub(crate) fn transition(t: &wire::Transition) -> Option<Transition> {
    Some(match t {
        wire::Transition::Instant(_) => Transition::Instant,
        wire::Transition::Spring(s) => Transition::Spring(spring(s)),
        wire::Transition::Tween(t) => Transition::Tween {
            duration_ms: t.duration_ms,
            ease: ease(&t.ease, &t.bezier)?,
        },
        wire::Transition::Token(t) => {
            Transition::Token(MotionToken::from_name(t.token.as_str()).unwrap_or(MotionToken::Ui))
        }
    })
}

fn key_ease(e: &Option<wire::KeyEase>) -> Option<KeyEase> {
    Some(match e {
        None => KeyEase::default(),
        Some(wire::KeyEase::Linear(_)) => KeyEase::Linear,
        Some(wire::KeyEase::Ease(e)) => KeyEase::Ease(ease(&e.ease, &[])?),
        Some(wire::KeyEase::Bezier(b)) => KeyEase::Ease(ease("", &b.bezier)?),
        Some(wire::KeyEase::Spring(s)) => KeyEase::Spring(spring(s)),
        Some(wire::KeyEase::Hold(_)) => KeyEase::Hold,
    })
}

fn event_kind(theme: &Theme, a: &wire::TimelineAction) -> EventKind {
    match a {
        wire::TimelineAction::Sound(e) => EventKind::Sound {
            name: e.name.clone(),
            pitch: e.playback_rate,
        },
        wire::TimelineAction::Haptic(e) => EventKind::Haptic(e.name.clone()),
        wire::TimelineAction::Burst(e) => EventKind::Burst {
            count: e.count,
            radius: e.radius,
            color: paint(
                theme,
                &json!({ "color": e.color.as_ref().map(ToJson::to_json) }),
            ),
            duration_ms: 0.0,
        },
        wire::TimelineAction::State(e) => EventKind::State {
            group: e.group.clone(),
            state: e.state.clone(),
        },
        wire::TimelineAction::Callback(e) => EventKind::Callback(e.name.clone()),
    }
}

/// A `UiTimeline` desc.
pub(crate) fn timeline(theme: &Theme, d: &wire::UiTimeline) -> Timeline {
    Timeline {
        tracks: d
            .tracks
            .iter()
            .filter_map(|t| {
                Some(TimelineTrack {
                    target: t.target.clone(),
                    prop: Prop::from_name(t.prop.as_str())?,
                    keys: t
                        .keys
                        .iter()
                        .filter_map(|k| {
                            Some(Key {
                                at_ms: k.at_ms,
                                value: k.value,
                                ease: key_ease(&k.ease)?,
                            })
                        })
                        .collect(),
                })
            })
            .collect(),
        events: d
            .events
            .iter()
            .map(|e| TimelineEvent {
                at_ms: e.at_ms,
                target: e.target.clone(),
                kind: event_kind(theme, &e.action),
            })
            .collect(),
        labels: d.labels.iter().map(|l| (l.name.clone(), l.at_ms)).collect(),
        duration_ms: d.duration_ms,
    }
}

fn stagger_from(s: &str) -> StaggerFrom {
    match s {
        "end" => StaggerFrom::End,
        "center" => StaggerFrom::Center,
        s => match s.split_once(',') {
            Some((x, y)) => match (x.trim().parse(), y.trim().parse()) {
                (Ok(x), Ok(y)) => StaggerFrom::Point(x, y),
                _ => StaggerFrom::Start,
            },
            None => StaggerFrom::Start,
        },
    }
}

/// A `Choreo` union.
pub(crate) fn choreo(c: &wire::Choreo) -> Choreo {
    let steps = |s: &[wire::Choreo]| s.iter().map(choreo).collect();
    match c {
        wire::Choreo::Play(p) => Choreo::Play {
            timeline: p.timeline.clone(),
            target: p.target.clone(),
            delay_ms: p.delay_ms,
        },
        wire::Choreo::Sequence(s) => Choreo::Sequence(steps(&s.steps)),
        wire::Choreo::Parallel(s) => Choreo::Parallel(steps(&s.steps)),
        wire::Choreo::Stagger(s) => Choreo::Stagger {
            timeline: s.timeline.clone(),
            targets: s.targets.clone(),
            step_ms: s.step_ms,
            cap_ms: s.cap_ms,
            from: stagger_from(&s.from),
        },
        wire::Choreo::Cascade(c) => Choreo::Cascade {
            steps: steps(&c.steps),
            on: c.on.clone(),
        },
    }
}

fn play_desc(d: &wire::PlayDesc) -> PlayDesc {
    PlayDesc {
        reverse: d.reverse,
        repeat: match d.repeat {
            wire::Repeat::Loop => Repeat::Loop,
            wire::Repeat::PingPong => Repeat::PingPong,
            wire::Repeat::Once => Repeat::Once,
        },
        speed: d.speed,
        from_ms: d.from_ms,
        from_label: d.from_label.clone(),
        hold: d.hold,
        decorative: d.decorative,
    }
}

/// A PlayDesc argument; a refused one is the default.
fn play_arg(desc: Option<String>) -> PlayDesc {
    play_desc(&wire::arg(desc.as_deref(), "desc").unwrap_or_default())
}

pub fn og_ui_define_timeline(name: String, timeline_desc: Option<String>) {
    let d: wire::UiTimeline = wire::arg(timeline_desc.as_deref(), "timeline").unwrap_or_default();
    with_ui("og.ui.defineTimeline", |ui| {
        let t = timeline(ui.theme(), &d);
        ui.define_timeline(&name, t)
    });
}

pub fn og_ui_play(path: String, choreo_desc: String, desc: Option<String>) -> u32 {
    let Ok(c) = wire::arg::<Option<wire::Choreo>>(Some(&choreo_desc), "choreo") else {
        return 0;
    };
    let Some(c) = c.as_ref().map(choreo) else {
        return 0;
    };
    let d = play_arg(desc);
    with_ui("og.ui.play", |ui| ui.play(&path, c, d)).unwrap_or(0)
}

pub fn og_ui_play_timeline(path: String, timeline: String, desc: Option<String>) -> u32 {
    let d = play_arg(desc);
    with_ui("og.ui.playTimeline", |ui| {
        ui.play_timeline(&path, &timeline, d)
    })
    .unwrap_or(0)
}

pub fn og_ui_seek(play: u32, ms: f64) -> bool {
    with_ui("og.ui.seek", |ui| ui.seek(play, ms as f32)).unwrap_or(false)
}

pub fn og_ui_pause(play: u32) -> bool {
    with_ui("og.ui.pause", |ui| ui.pause(play)).unwrap_or(false)
}

pub fn og_ui_resume(play: u32) -> bool {
    with_ui("og.ui.resume", |ui| ui.resume(play)).unwrap_or(false)
}

pub fn og_ui_reverse(play: u32) -> bool {
    with_ui("og.ui.reverse", |ui| ui.reverse(play)).unwrap_or(false)
}

pub fn og_ui_stop(play: u32) -> bool {
    with_ui("og.ui.stop", |ui| ui.stop(play)).unwrap_or(false)
}

pub fn og_ui_play_info(play: u32) -> String {
    with_ui("og.ui.playInfo", |ui| ui.play_info(play))
        .flatten()
        .map_or(Value::Null, |p| {
            json!({
                "atMs": p.at_ms, "lengthMs": p.length_ms, "paused": p.paused,
                "reverse": p.reverse, "ended": p.ended,
            })
        })
        .to_string()
}

pub fn og_ui_force_state(path: String, group: String, state: String) {
    let s = wire::arg::<Option<String>>(Some(&state), "state").unwrap_or_default();
    with_ui("og.ui.forceState", |ui| {
        ui.force_state(&path, &group, s.as_deref())
    });
}

pub fn og_ui_visual_state(path: String, group: String) -> String {
    with_ui("og.ui.visualState", |ui| ui.visual_state(&path, &group))
        .flatten()
        .unwrap_or_default()
}

pub fn og_ui_animate(id: String, target: f64, transition_desc: String) -> f64 {
    let t = wire::arg::<Option<wire::Transition>>(Some(&transition_desc), "transition")
        .ok()
        .flatten()
        .and_then(|t| transition(&t))
        .unwrap_or_default();
    with_ui("og.ui.animate", |ui| ui.animate(&id, target as f32, t)).map_or(target, f64::from)
}

pub fn og_ui_gesture(path: String) -> String {
    let g = with_ui("og.ui.gesture", |ui| ui.gesture_at(&path)).unwrap_or_default();
    json!({
        "held": g.held, "dragging": g.dragging,
        "offsetX": g.offset_x, "offsetY": g.offset_y,
        "velocityX": g.velocity_x, "velocityY": g.velocity_y,
        "scale": g.scale, "rotation": g.rotation, "snapped": g.snapped,
        "dismissed": g.dismissed, "over": g.over, "droppedOn": g.dropped_on,
        "hovering": g.hovering, "received": g.received,
        "tapped": g.tapped, "doubleTapped": g.double_tapped,
        "longPressed": g.long_pressed, "longPress": g.long_press,
    })
    .to_string()
}

pub fn og_ui_motion_events() -> String {
    let events = with_ui("og.ui.motionEvents", |ui| {
        ui.events()
            .iter()
            .map(|e| {
                json!({
                    "kind": e.kind, "name": e.name, "target": e.target,
                    "play": e.play, "index": e.index, "playbackRate": e.pitch,
                })
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    Value::Array(events).to_string()
}

pub fn og_ui_overlaps() -> String {
    let all = with_ui("og.ui.overlaps", |ui| {
        ui.overlaps()
            .iter()
            .map(|o| {
                json!({
                    "path": o.path, "other": o.other, "rect": super::rect_json(&o.rect),
                    "inFlow": o.in_flow,
                })
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    Value::Array(all).to_string()
}

pub fn og_ui_layout_stats() -> String {
    let s = with_ui("og.ui.layoutStats", |ui| ui.frame().stats).unwrap_or_default();
    json!({
        "elements": s.elements, "rebuilt": s.rebuilt, "laidOut": s.laid_out,
        "keptCommands": s.kept_commands,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_choreography_nests() {
        let c = choreo(
            &wire::arg(
                Some(
                    r#"{ "kind": "sequence", "steps": [
                { "kind": "play", "timeline": "in" },
                { "kind": "stagger", "timeline": "pop", "stepMs": 60, "capMs": 300, "from": "center" },
                { "kind": "cascade", "on": "count", "steps": [{ "kind": "play", "timeline": "a" }] }
            ] }"#,
                ),
                "choreo",
            )
            .expect("a choreo"),
        );
        let Choreo::Sequence(steps) = c else {
            panic!("{c:?}")
        };
        assert_eq!(steps.len(), 3);
        assert!(matches!(
            steps[1],
            Choreo::Stagger {
                cap_ms: 300.0,
                from: StaggerFrom::Center,
                ..
            }
        ));
        assert!(
            matches!(&steps[2], Choreo::Cascade { on, steps } if on == "count" && steps.len() == 1)
        );
        assert_eq!(stagger_from("10, 20"), StaggerFrom::Point(10.0, 20.0));
    }
}
