//! `og.ui`'s motion (docs/ui.md §3.1): the JSON of `UiTimeline`, `Choreo`
//! and `Transition` read into og-ui's types, and the calls that take them.

use super::{f, paint, with_ui};
use og_ui::{
    Choreo, Ease, EventKind, Key, KeyEase, MotionToken, PlayDesc, Prop, Repeat, SpringSpec,
    StaggerFrom, Theme, Timeline, TimelineEvent, TimelineTrack, Transition,
};
use serde_json::{Value, json};

fn list(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

fn string(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_owned()
}

fn strings(v: &Value) -> Vec<String> {
    list(v)
        .iter()
        .filter_map(|s| s.as_str().map(str::to_owned))
        .collect()
}

fn prop(v: &Value) -> Option<Prop> {
    Prop::from_name(v.as_str()?)
}

fn spring(v: &Value) -> SpringSpec {
    SpringSpec::new(f(v, "durationMs", 350.0), f(v, "bounce", 0.0))
}

fn ease(v: &Value, bezier: &Value) -> Option<Ease> {
    if !list(bezier).is_empty() {
        let [a, b, c, d] = list(bezier) else {
            return None;
        };
        let curve = Ease::CubicBezier([
            a.as_f64()? as f32,
            b.as_f64()? as f32,
            c.as_f64()? as f32,
            d.as_f64()? as f32,
        ]);
        return curve.is_valid().then_some(curve);
    }
    if v.is_null() {
        Some(Ease::CubicBezier(og_ui::theme::EASE_OUT))
    } else {
        Ease::from_text(v.as_str()?)
    }
}

fn token(v: &Value) -> MotionToken {
    v.as_str()
        .and_then(MotionToken::from_name)
        .unwrap_or(MotionToken::Ui)
}

/// A `Transition` union; `None` for null or an unknown kind.
pub(crate) fn transition(v: &Value) -> Option<Transition> {
    Some(match v["kind"].as_str()? {
        "instant" => Transition::Instant,
        "spring" => Transition::Spring(spring(v)),
        "tween" => Transition::Tween {
            duration_ms: f(v, "durationMs", 200.0),
            ease: ease(&v["ease"], &v["bezier"])?,
        },
        "token" => Transition::Token(token(&v["token"])),
        _ => return None,
    })
}

fn key_ease(v: &Value) -> Option<KeyEase> {
    Some(match v["kind"].as_str() {
        Some("linear") => KeyEase::Linear,
        Some("ease") => KeyEase::Ease(ease(&v["ease"], &Value::Null)?),
        Some("bezier") => KeyEase::Ease(ease(&Value::Null, &v["bezier"])?),
        Some("spring") => KeyEase::Spring(spring(v)),
        Some("hold") => KeyEase::Hold,
        _ => KeyEase::default(),
    })
}

fn event_kind(theme: &Theme, v: &Value) -> Option<EventKind> {
    Some(match v["kind"].as_str()? {
        "sound" => EventKind::Sound {
            name: string(&v["name"]),
            pitch: f(v, "playbackRate", 1.0),
        },
        "haptic" => EventKind::Haptic(string(&v["name"])),
        "burst" => EventKind::Burst {
            count: v["count"].as_u64().map_or(0, |n| n as u32),
            radius: f(v, "radius", 0.0),
            color: paint(theme, &json!({ "color": v["color"] })),
            duration_ms: 0.0,
        },
        "state" => EventKind::State {
            group: string(&v["group"]),
            state: string(&v["state"]),
        },
        "callback" => EventKind::Callback(string(&v["name"])),
        _ => return None,
    })
}

/// A `UiTimeline` desc.
pub(crate) fn timeline(theme: &Theme, d: &Value) -> Timeline {
    Timeline {
        tracks: list(&d["tracks"])
            .iter()
            .filter_map(|t| {
                Some(TimelineTrack {
                    target: string(&t["target"]),
                    prop: prop(&t["prop"])?,
                    keys: list(&t["keys"])
                        .iter()
                        .filter_map(|k| {
                            Some(Key {
                                at_ms: f(k, "atMs", 0.0),
                                value: f(k, "value", 0.0),
                                ease: key_ease(&k["ease"])?,
                            })
                        })
                        .collect(),
                })
            })
            .collect(),
        events: list(&d["events"])
            .iter()
            .filter_map(|e| {
                Some(TimelineEvent {
                    at_ms: f(e, "atMs", 0.0),
                    target: string(&e["target"]),
                    kind: event_kind(theme, &e["action"])?,
                })
            })
            .collect(),
        labels: list(&d["labels"])
            .iter()
            .map(|l| (string(&l["name"]), f(l, "atMs", 0.0)))
            .collect(),
        duration_ms: f(d, "durationMs", 0.0),
    }
}

fn stagger_from(v: &Value) -> StaggerFrom {
    match v.as_str().unwrap_or("start") {
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
pub(crate) fn choreo(v: &Value) -> Option<Choreo> {
    let steps = |v: &Value| list(&v["steps"]).iter().filter_map(choreo).collect();
    Some(match v["kind"].as_str()? {
        "play" => Choreo::Play {
            timeline: string(&v["timeline"]),
            target: string(&v["target"]),
            delay_ms: f(v, "delayMs", 0.0),
        },
        "sequence" => Choreo::Sequence(steps(v)),
        "parallel" => Choreo::Parallel(steps(v)),
        "stagger" => Choreo::Stagger {
            timeline: string(&v["timeline"]),
            targets: strings(&v["targets"]),
            step_ms: f(v, "stepMs", 0.0),
            cap_ms: f(v, "capMs", 0.0),
            from: stagger_from(&v["from"]),
        },
        "cascade" => Choreo::Cascade {
            steps: steps(v),
            on: string(&v["on"]),
        },
        _ => return None,
    })
}

fn play_desc(d: &Value) -> PlayDesc {
    PlayDesc {
        reverse: d["reverse"].as_bool().unwrap_or(false),
        repeat: match d["repeat"].as_str() {
            Some("loop") => Repeat::Loop,
            Some("pingPong") => Repeat::PingPong,
            _ => Repeat::Once,
        },
        speed: f(d, "speed", 0.0),
        from_ms: f(d, "fromMs", 0.0),
        from_label: string(&d["fromLabel"]),
        hold: d["hold"].as_bool().unwrap_or(false),
        decorative: d["decorative"].as_bool().unwrap_or(false),
    }
}

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

pub fn og_ui_define_timeline(name: String, timeline_desc: Option<String>) {
    let d = super::desc_of(timeline_desc);
    with_ui("og.ui.defineTimeline", |ui| {
        let t = timeline(ui.theme(), &d);
        ui.define_timeline(&name, t)
    });
}

pub fn og_ui_play(path: String, choreo_desc: String, desc: Option<String>) -> u32 {
    let d = super::desc_of(desc);
    let Some(c) = choreo(&parse(&choreo_desc)) else {
        return 0;
    };
    with_ui("og.ui.play", |ui| ui.play(&path, c, play_desc(&d))).unwrap_or(0)
}

pub fn og_ui_play_timeline(path: String, timeline: String, desc: Option<String>) -> u32 {
    let d = super::desc_of(desc);
    with_ui("og.ui.playTimeline", |ui| {
        ui.play_timeline(&path, &timeline, play_desc(&d))
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
    let s = parse(&state);
    with_ui("og.ui.forceState", |ui| {
        ui.force_state(&path, &group, s.as_str())
    });
}

pub fn og_ui_visual_state(path: String, group: String) -> String {
    with_ui("og.ui.visualState", |ui| ui.visual_state(&path, &group))
        .flatten()
        .unwrap_or_default()
}

pub fn og_ui_animate(id: String, target: f64, transition_desc: String) -> f64 {
    let t = transition(&parse(&transition_desc)).unwrap_or_default();
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
        let c = choreo(&parse(
            r#"{ "kind": "sequence", "steps": [
                { "kind": "play", "timeline": "in" },
                { "kind": "stagger", "timeline": "pop", "stepMs": 60, "capMs": 300, "from": "center" },
                { "kind": "cascade", "on": "count", "steps": [{ "kind": "play", "timeline": "a" }] }
            ] }"#,
        ))
        .expect("a choreo");
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
        assert_eq!(
            stagger_from(&json!("10, 20")),
            StaggerFrom::Point(10.0, 20.0)
        );
    }
}
