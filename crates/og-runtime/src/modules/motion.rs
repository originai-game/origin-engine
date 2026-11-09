//! `og.ui`'s element model and motion (docs/ui.md §2, §3.1): the JSON of
//! `ElementProps`, `UiTimeline`, `Choreo` and `Transition` read into
//! og-ui's types, and the calls that take them.

use super::{
    anchor_of, customize, f, insets, opt_f, paint, size, transition_kind, ui_rect, with_ui,
};
use og_ui::{
    Breakpoint, Choreo, Dismiss, DragAxis, DragSpec, DropSpec, Ease, Edges, EventKind, GridFlow,
    GridPlace, GridSpec, Key, KeyEase, LayoutMode, MotionToken, Orientation, Patch, PinchSpec,
    PlayDesc, Position, PresenceSpec, Prop, Props, Query, Reduced, Repeat, SpringSpec, StaggerFrom,
    StateGroup, StateTransition, Theme, Timeline, TimelineEvent, TimelineTrack, Track, Transform,
    Transition, Variant, VisualState,
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

fn u16_(v: &Value, k: &str) -> u16 {
    v[k].as_u64().map_or(0, |n| n.min(u16::MAX as u64) as u16)
}

fn transform(v: &Value) -> Transform {
    Transform {
        x: f(v, "x", 0.0),
        y: f(v, "y", 0.0),
        scale_x: f(v, "scaleX", 1.0),
        scale_y: f(v, "scaleY", 1.0),
        rotate: f(v, "rotate", 0.0),
        skew_x: f(v, "skewX", 0.0),
        skew_y: f(v, "skewY", 0.0),
        pivot_x: f(v, "pivotX", 0.5),
        pivot_y: f(v, "pivotY", 0.5),
    }
}

fn edges(v: &Value) -> Edges {
    Edges {
        start: opt_f(v, "start"),
        top: opt_f(v, "top"),
        end: opt_f(v, "end"),
        bottom: opt_f(v, "bottom"),
        center_x: v["centerX"].as_bool().unwrap_or(false),
        center_y: v["centerY"].as_bool().unwrap_or(false),
        across: None,
    }
}

fn track(v: &Value) -> Track {
    match v["kind"].as_str() {
        Some("px") => Track::Px(f(v, "px", 0.0)),
        Some("fr") => Track::Fr(f(v, "fr", 1.0)),
        Some("minMax") => Track::MinMax {
            min: f(v, "min", 0.0),
            max: f(v, "max", f32::INFINITY),
        },
        Some("percent") => Track::Percent(f(v, "fraction", 0.0)),
        _ => Track::Auto,
    }
}

fn tracks(v: &Value) -> Vec<Track> {
    list(v).iter().map(track).collect()
}

fn grid(v: &Value) -> GridSpec {
    GridSpec {
        columns: tracks(&v["columns"]),
        rows: tracks(&v["rows"]),
        auto_track: if v["autoTrack"].is_object() {
            track(&v["autoTrack"])
        } else {
            Track::Auto
        },
        column_gap: opt_f(v, "columnGap"),
        row_gap: opt_f(v, "rowGap"),
        areas: strings(&v["areas"]),
        flow: match v["flow"].as_str() {
            Some("column") => GridFlow::Column,
            _ => GridFlow::Row,
        },
        dense: v["dense"].as_bool().unwrap_or(false),
        ..GridSpec::default()
    }
}

fn place(v: &Value) -> GridPlace {
    GridPlace {
        column: u16_(v, "column"),
        row: u16_(v, "row"),
        column_span: u16_(v, "columnSpan"),
        row_span: u16_(v, "rowSpan"),
        area: string(&v["area"]),
        ..GridPlace::default()
    }
}

fn prop(v: &Value) -> Option<Prop> {
    Prop::from_name(v.as_str()?)
}

fn values(v: &Value) -> Vec<(Prop, f32)> {
    list(v)
        .iter()
        .filter_map(|p| Some((prop(&p["prop"])?, f(p, "value", 0.0))))
        .collect()
}

fn breakpoint(v: &Value) -> Option<Breakpoint> {
    Some(match v.as_str()? {
        "phonePortrait" => Breakpoint::PhonePortrait,
        "phoneLandscape" => Breakpoint::PhoneLandscape,
        "tablet" => Breakpoint::Tablet,
        "desktop" => Breakpoint::Desktop,
        _ => return None,
    })
}

fn variant(v: &Value) -> Variant {
    let w = &v["when"];
    let s = &v["set"];
    let opt_bool = |k: &str| s[k].as_bool();
    Variant {
        when: Query {
            breakpoints: list(&w["breakpoints"])
                .iter()
                .filter_map(breakpoint)
                .collect(),
            orientation: match w["orientation"].as_str() {
                Some("portrait") => Some(Orientation::Portrait),
                Some("landscape") => Some(Orientation::Landscape),
                _ => None,
            },
            min_width: f(w, "minWidth", 0.0),
            max_width: f(w, "maxWidth", 0.0),
            container: w["container"].as_bool().unwrap_or(false),
        },
        set: Patch {
            direction: match s["direction"].as_str() {
                Some("row") => Some(og_ui::Direction::Row),
                Some("column") => Some(og_ui::Direction::Column),
                _ => None,
            },
            width: size(&s["width"]),
            height: size(&s["height"]),
            padding: s["padding"].is_object().then(|| insets(&s["padding"])),
            gap: opt_f(s, "gap"),
            align: s["align"].is_string().then(|| super::align(&s["align"])),
            cross: s["cross"].is_string().then(|| super::cross(&s["cross"])),
            wrap: opt_bool("wrap"),
            layout: s["layout"].is_string().then(|| layout_mode(&s["layout"])),
            columns: s["columns"].is_array().then(|| tracks(&s["columns"])),
            hidden: opt_bool("hidden"),
            values: values(&s["values"]),
        },
    }
}

fn layout_mode(v: &Value) -> LayoutMode {
    match v.as_str() {
        Some("stack") => LayoutMode::Stack,
        Some("grid") => LayoutMode::Grid,
        _ => LayoutMode::Flex,
    }
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

fn presence(v: &Value, exit: bool) -> Option<PresenceSpec> {
    if !v.is_object() {
        return None;
    }
    let t = transition(&v["transition"]).unwrap_or(Transition::Token(if exit {
        MotionToken::Exit
    } else {
        MotionToken::Enter
    }));
    let mut p = match transition_kind(&v["kind"]) {
        Some(kind) => PresenceSpec::from_kind(kind, f(v, "distance", 24.0), t),
        None => PresenceSpec {
            values: values(&v["values"]),
            transition: t,
            delay_ms: 0.0,
        },
    };
    p.delay_ms = f(v, "delayMs", 0.0);
    Some(p)
}

fn drag(v: &Value) -> Option<DragSpec> {
    if !v.is_object() {
        return None;
    }
    Some(DragSpec {
        axis: match v["axis"].as_str() {
            Some("x") => DragAxis::X,
            Some("y") => DragAxis::Y,
            Some("lock") => DragAxis::Lock,
            _ => DragAxis::Both,
        },
        bounds: v["bounds"].is_object().then(|| ui_rect(&v["bounds"])),
        rubber: f(v, "rubber", 0.0),
        snap: list(&v["snap"])
            .iter()
            .map(|p| (f(p, "x", 0.0), f(p, "y", 0.0)))
            .collect(),
        momentum: v["momentum"].as_bool().unwrap_or(false),
        return_home: v["returnHome"].as_bool().unwrap_or(false),
        dismiss: v["dismiss"].is_object().then(|| Dismiss {
            distance: f(&v["dismiss"], "distance", 0.0),
            velocity: f(&v["dismiss"], "velocity", 0.0),
        }),
        threshold: f(v, "threshold", 0.0),
        spring: transition(&v["spring"]),
        tag: string(&v["tag"]),
    })
}

fn states(v: &Value) -> Vec<StateGroup> {
    list(v)
        .iter()
        .map(|g| StateGroup {
            name: string(&g["name"]),
            states: list(&g["states"])
                .iter()
                .map(|s| VisualState {
                    name: string(&s["name"]),
                    values: values(&s["values"]),
                    on_enter: string(&s["onEnter"]),
                    on_exit: string(&s["onExit"]),
                })
                .collect(),
            transitions: list(&g["transitions"])
                .iter()
                .map(|t| StateTransition {
                    from: string(&t["from"]),
                    to: string(&t["to"]),
                    transition: transition(&t["transition"]).unwrap_or_default(),
                })
                .collect(),
        })
        .collect()
}

/// An `ElementProps` desc.
pub(crate) fn props(d: &Value) -> Props {
    let z = Props::default();
    Props {
        transform: if d["transform"].is_object() {
            transform(&d["transform"])
        } else {
            z.transform
        },
        space3d: None,
        opacity: f(d, "opacity", 1.0).clamp(0.0, 1.0),
        z: d["z"].as_i64().map_or(0, |n| n as i32),
        position: match d["position"].as_str() {
            Some("absolute") => Position::Absolute,
            Some("screen") => Position::Screen,
            Some("anchored") => Position::Anchored,
            _ => Position::Flow,
        },
        insets: edges(&d["insets"]),
        anchor_to: string(&d["anchorTo"]),
        anchor_at: anchor_of(&d["anchorAt"]).unwrap_or_default(),
        anchor_self: anchor_of(&d["anchorSelf"]).unwrap_or_default(),
        hit_slop: f(d, "hitSlop", 0.0).max(0.0),
        layout: layout_mode(&d["layout"]),
        grid: grid(&d["grid"]),
        place: place(&d["place"]),
        container: d["container"].as_bool().unwrap_or(false),
        variants: list(&d["variants"]).iter().map(variant).collect(),
        overlaps: strings(&d["overlaps"]),
        dense: d["dense"].as_bool().unwrap_or(false),
        match_key: string(&d["match"]),
        drag: drag(&d["drag"]),
        pinch: d["pinch"].is_object().then(|| PinchSpec {
            min_scale: f(&d["pinch"], "minScale", 0.0),
            max_scale: f(&d["pinch"], "maxScale", 0.0),
            rotate: d["pinch"]["rotate"].as_bool().unwrap_or(false),
        }),
        drop: d["drop"].is_object().then(|| DropSpec {
            accept: strings(&d["drop"]["accept"]),
        }),
        // Documents' pointer handlers only; built UI's interactive flag hits.
        pointer: false,
        // Documents' SVG text only.
        view_place: None,
        nav: og_ui::NavLinks {
            up: string(&d["nav"]["up"]),
            down: string(&d["nav"]["down"]),
            left: string(&d["nav"]["left"]),
            right: string(&d["nav"]["right"]),
            next: string(&d["nav"]["next"]),
            prev: string(&d["nav"]["prev"]),
        },
        focus_group: d["focusGroup"].is_object().then(|| og_ui::FocusGroup {
            wrap: d["focusGroup"]["wrap"].as_bool().unwrap_or(false),
            memory: d["focusGroup"]["memory"].as_bool().unwrap_or(false),
        }),
        gestures: d["gestures"].is_object().then(|| {
            let g = &d["gestures"];
            og_ui::GestureSpec {
                tap: g["tap"].as_bool().unwrap_or(false),
                double_tap: g["doubleTap"].as_bool().unwrap_or(false),
                long_press: g["longPress"].as_bool().unwrap_or(false),
                long_press_ms: f(g, "longPressMs", 0.0).max(0.0),
                double_tap_ms: f(g, "doubleTapMs", 0.0).max(0.0),
                slop: f(g, "slop", 0.0).max(0.0),
                priority: g["priority"].as_i64().map_or(0, |n| n as i32),
            }
        }),
        cursor: d["cursor"]
            .as_str()
            .and_then(og_ui::Cursor::from_name)
            .unwrap_or_default(),
        cursor_images: Vec::new(),
        interact: og_ui::Interact::default(),
        text_path: text_path(&d["textPath"]),
        player: d["player"].as_u64().map(|k| k.min(3) as u8),
        select: d["userSelect"]
            .as_str()
            .and_then(og_ui::UserSelect::from_name)
            .unwrap_or_default(),
        customize: customize(&d["customize"]),
        motion: transition(&d["motion"]),
        layout_motion: transition(&d["layoutMotion"]),
        // CSS transitions and animations come from documents only.
        transitions: z.transitions,
        animations: z.animations,
        enter: presence(&d["enter"], false),
        starting: z.starting,
        exit: presence(&d["exit"], true),
        states: states(&d["states"]),
        state: list(&d["state"])
            .iter()
            .map(|s| (string(&s["group"]), string(&s["state"])))
            .collect(),
        reduced: match d["reduced"].as_str() {
            Some("fade") => Some(Reduced::Fade),
            Some("none") => Some(Reduced::None),
            Some("keep") => Some(Reduced::Keep),
            _ => None,
        },
        inert: d["inert"].as_bool().unwrap_or(false),
    }
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

pub fn og_ui_props(desc: Option<String>) -> bool {
    let d = super::desc_of(desc);
    with_ui("og.ui.props", |ui| ui.set_props(props(&d))).unwrap_or(false)
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

pub fn og_ui_open_path() -> String {
    with_ui("og.ui.openPath", |ui| ui.open_path()).unwrap_or_default()
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

pub fn og_ui_semantics(desc: Option<String>) -> bool {
    let d = super::desc_of(desc);
    let text = |k: &str| d[k].as_str().map(str::to_owned);
    let num = |k: &str| d[k].as_f64().map(|v| v as f32);
    let s = og_ui::Semantics {
        role: text("role"),
        name: text("name"),
        description: text("description"),
        value: text("value"),
        checked: d["checked"].as_bool(),
        selected: d["selected"].as_bool(),
        expanded: d["expanded"].as_bool(),
        busy: d["busy"].as_bool().unwrap_or(false),
        disabled: d["disabled"].as_bool().unwrap_or(false),
        value_min: num("valueMin"),
        value_max: num("valueMax"),
        value_now: num("valueNow"),
    };
    with_ui("og.ui.semantics", |ui| ui.semantics(&s)).unwrap_or(false)
}

/// `ElementProps.textPath`: its path data, or a circle; `None` (text in
/// lines) for null or data that does not parse.
fn text_path(v: &Value) -> Option<og_ui::TextPath> {
    if !v.is_object() {
        return None;
    }
    let data = v["data"].as_str().unwrap_or("");
    let shape = if data.trim().is_empty() {
        let r = v["radius"].as_f64().unwrap_or(0.0) as f32;
        og_ui::PathShape::Circle((r > 0.0).then_some(r))
    } else {
        og_ui::text_path::parse_shape(&format!("path({})", serde_json::json!(data))).ok()??
    };
    Some(og_ui::TextPath {
        shape,
        offset: v["offset"].as_f64().unwrap_or(0.0) as f32,
        percent: v["percent"].as_bool().unwrap_or(false),
        reverse: v["reverse"].as_bool().unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn props_read_every_part_of_their_desc() {
        let d = parse(
            r#"{
            "transform": { "x": 4, "rotate": 0.5, "pivotX": 0 },
            "opacity": 0.5, "z": 3, "position": "absolute",
            "insets": { "end": 8, "top": 2 },
            "layout": "grid",
            "grid": { "columns": [{ "kind": "px", "px": 100 }, { "kind": "fr", "fr": 2 }],
                      "areas": ["a b"] },
            "place": { "area": "b" },
            "variants": [{ "when": { "breakpoints": ["phonePortrait"] },
                           "set": { "columns": [{ "kind": "fr", "fr": 1 }], "hidden": true } }],
            "match": "hero",
            "drag": { "axis": "x", "snap": [{ "x": 0, "y": 0 }, { "x": 200, "y": 0 }],
                      "momentum": true, "spring": { "kind": "spring", "durationMs": 300, "bounce": 0.2 } },
            "motion": { "kind": "token", "token": "playful" },
            "exit": { "kind": "slideUp", "distance": 30 },
            "states": [{ "name": "selection",
                         "states": [{ "name": "off", "values": [] },
                                    { "name": "on", "values": [{ "prop": "scale", "value": 1.2 }] }],
                         "transitions": [{ "from": "*", "to": "on",
                                           "transition": { "kind": "tween", "durationMs": 120, "ease": "none" } }] }],
            "state": [{ "group": "selection", "state": "on" }]
        }"#,
        );
        let p = props(&d);
        assert_eq!(p.transform.x, 4.0);
        assert_eq!(p.transform.pivot_x, 0.0);
        assert_eq!(
            p.transform.scale_x, 1.0,
            "absent fields keep their defaults"
        );
        assert_eq!((p.opacity, p.z, p.position), (0.5, 3, Position::Absolute));
        assert_eq!((p.insets.end, p.insets.start), (Some(8.0), None));
        assert_eq!(p.grid.columns, [Track::Px(100.0), Track::Fr(2.0)]);
        assert_eq!(p.place.area, "b");
        assert_eq!(p.variants[0].when.breakpoints, [Breakpoint::PhonePortrait]);
        assert_eq!(p.variants[0].set.hidden, Some(true));
        assert_eq!(p.match_key, "hero");
        let drag = p.drag.expect("drag");
        assert_eq!((drag.axis, drag.snap.len()), (DragAxis::X, 2));
        assert_eq!(
            drag.spring,
            Some(Transition::Spring(SpringSpec::new(300.0, 0.2)))
        );
        assert_eq!(p.motion, Some(Transition::Token(MotionToken::Playful)));
        let exit = p.exit.expect("exit");
        assert_eq!(exit.values, [(Prop::Opacity, 0.0), (Prop::Y, 30.0)]);
        assert_eq!(exit.transition, Transition::Token(MotionToken::Exit));
        assert_eq!(p.states[0].states[1].values, [(Prop::Scale, 1.2)]);
        assert_eq!(
            p.states[0].transition("off", "on"),
            Some(Transition::Tween {
                duration_ms: 120.0,
                ease: Ease::Linear
            })
        );
        assert_eq!(p.state, [("selection".to_owned(), "on".to_owned())]);
        for (name, expected) in [("power1.in", 0.0625), ("power1.out", 0.4375)] {
            let p = props(&json!({"motion": {"kind":"tween", "ease":name}}));
            let Some(Transition::Tween { ease, .. }) = p.motion else {
                panic!("missing {name}")
            };
            assert_eq!(ease.apply(0.25), expected);
        }
        for name in ["linear", "out", "exit", "inOut", "ease-out"] {
            assert!(
                props(&json!({"motion": {"kind":"tween", "ease":name}}))
                    .motion
                    .is_none(),
                "removed or CSS name {name} is not a GSAP alias"
            );
        }
    }

    #[test]
    fn a_text_path_desc_reads_as_its_css_does() {
        let p = props(
            &json!({"textPath": {"data": "M0 10 h100", "offset": 0.5, "percent": true, "reverse": true}}),
        );
        let tp = p.text_path.expect("a path");
        assert_eq!(tp.extent(), Some(og_core::glam::Vec2::new(100.0, 10.0)));
        assert!((tp.offset, tp.percent, tp.reverse) == (0.5, true, true));
        // Empty data is a circle: its radius, or none for the box's side.
        let circle = |d| props(&d).text_path.map(|t| t.shape);
        assert_eq!(
            circle(json!({"textPath": {"radius": 40}})),
            Some(og_ui::PathShape::Circle(Some(40.0)))
        );
        assert_eq!(
            circle(json!({"textPath": {}})),
            Some(og_ui::PathShape::Circle(None))
        );
        // Null, and data that does not parse, keep the text in lines.
        assert_eq!(circle(json!({"textPath": null})), None);
        assert_eq!(circle(json!({"textPath": {"data": "L1 1"}})), None);
    }

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
