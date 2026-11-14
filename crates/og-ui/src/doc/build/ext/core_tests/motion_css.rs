//! Documents driving transitions and animations tick by tick.
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
<panel id="Card" class:on={{$on}} class:paused={{$paused}}/>
<style>{style}</style>"#
    );
    ui.define_document(UiDocument::parse("m.svelte", &text).unwrap());
    ui.push_screen("m", json!({"on": false, "paused": false}))
        .unwrap();
    ui
}

/// One tick with `vm`'s values in the screen's stores; answers the card's
/// channel values.
fn tick(ui: &mut Ui, vm: &Value) -> [f32; crate::element::CHANNELS] {
    let mount = crate::layers::pushed("m");
    let inputs = ui.docs.state().memory.retained.registrations[&mount]
        .inputs
        .clone();
    for (name, value) in vm.as_object().unwrap() {
        ui.docs
            .execute(crate::reactive::session::Command::Set {
                source: inputs[name].clone(),
                value: value.clone(),
                force: false,
            })
            .unwrap();
    }
    ui.begin(InputFrame::default(), Viewport::new(640.0, 480.0));
    ui.end();
    let i = (0..ui.tree.nodes().len())
        .find(|&i| ui.tree.path_of(i) == "m/Card")
        .expect("the card");
    crate::layout::read_values(&ui.tree.nodes()[i])
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn a_linear_easing_times_a_transition_by_its_points_and_restores() {
    // 60 ticks a second: 300 ms is 18 ticks; linear(0, 0.8 25%, 1)
    // reaches 0.8 of the way a quarter through (4.5 ticks).
    let mut ui = ui(
        "#Card { width: 40px; height: 20px; transition: translate 300ms linear(0, 0.8 25%, 1); }
         #Card.on { translate: 100px 0; }",
        false,
    );
    let on = json!({"on": true, "paused": false});
    tick(&mut ui, &json!({"on": false, "paused": false}));
    let curve = |t: f32| {
        if t < 0.25 {
            3.2 * t
        } else {
            0.8 + (t - 0.25) / 0.75 * 0.2
        }
    };
    for j in 0..20 {
        let v = tick(&mut ui, &on);
        let t = (j as f32 / 18.0).min(1.0);
        assert!(close(v[X], 100.0 * curve(t)), "x at {j}: {}", v[X]);
        if j == 6 {
            let saved = ui.save_state();
            let rest: Vec<_> = (0..4).map(|_| tick(&mut ui, &on)).collect();
            ui.restore_state(&saved).unwrap();
            for r in rest {
                assert_eq!(tick(&mut ui, &on), r, "after restore");
            }
            // Back to tick 6 for the rest of the run.
            ui.restore_state(&saved).unwrap();
        }
    }
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
        "#Card { width: 40px; height: 20px; transition: opacity 400ms linear;
                 animation: slide 600ms linear forwards, spin 300ms linear infinite; }
         #Card.on { opacity: 0.4; }
         @media (prefers-reduced-motion: reduce) { #Card { transition: none; } }
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
        let i = (0..ui.tree.nodes().len())
            .find(|&i| ui.tree.path_of(i) == "m/Box/Card")
            .unwrap();
        crate::layout::read_values(&ui.tree.nodes()[i])[SCALE]
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

#[test]
fn computed_values_transition_reverse_shortened_animate_and_compose() {
    // 60 ticks a second: 100 ms is 6 ticks.
    let mut ui = ui(
        "#Card { width: 40px; height: 20px; transition: width 100ms linear, height 100ms linear; }
         #Card.on { width: 100px; }
         #Card.paused { animation: grow 100ms linear both, nudge 100ms linear both;
                        animation-composition: replace, add; }
         @keyframes grow { to { height: 50px; } }
         @keyframes nudge { from { height: 10px; } to { height: 30px; } }",
        false,
    );
    let size = |ui: &mut Ui, on: bool, paused: bool| {
        tick(ui, &json!({"on": on, "paused": paused}));
        let i = (0..ui.tree.nodes().len())
            .find(|&i| ui.tree.path_of(i) == "m/Card")
            .unwrap();
        (
            ui.tree.nodes()[i].placed.rect.width,
            ui.tree.nodes()[i].placed.rect.height,
        )
    };
    size(&mut ui, false, false);
    for j in 0..3 {
        let (w, _) = size(&mut ui, true, false);
        assert!(close(w, 40.0 + 60.0 * j as f32 / 6.0), "width at {j}: {w}");
    }
    // Reversed two thirds of the way, at 80 px: back to 40 px in two
    // thirds of the time (CSS Transitions 1 §3, the reversing
    // shortening factor).
    let (w, _) = size(&mut ui, true, false);
    assert!(close(w, 70.0), "{w}");
    for j in 0..6 {
        let (w, _) = size(&mut ui, false, false);
        assert!(
            close(w, 80.0 - 40.0 * (j as f32 / 4.0).min(1.0)),
            "back at {j}: {w}"
        );
    }
    assert!(
        ui.interaction
            .state()
            .motion
            .styles
            .values()
            .all(|s| s.runs.is_empty()),
        "settled transitions are not kept"
    );
    // The keyframes animate height; `add` composes onto grow's value.
    for j in 0..8 {
        let (_, h) = size(&mut ui, false, true);
        let k = (j as f32 / 6.0).min(1.0);
        let grow = 20.0 + 30.0 * k;
        // Layout snaps boxes to whole pixels.
        assert!(
            (h - (grow + 10.0 + 20.0 * k)).abs() <= 0.5,
            "height at {j}: {h}"
        );
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

#[test]
fn starting_style_is_the_first_styles_before_change_value() {
    // 60 ticks a second: 100 ms is 6 ticks.
    let mut ui = ui(
        "#Card { width: 40px; height: 20px; opacity: 1; transition: opacity 100ms linear, width 100ms linear; }
         @starting-style { #Card { opacity: 0; width: 10px; } }",
        false,
    );
    let off = json!({"on": false, "paused": false});
    for j in 0..9 {
        let v = tick(&mut ui, &off);
        let i = (0..ui.tree.nodes().len())
            .find(|&i| ui.tree.path_of(i) == "m/Card")
            .unwrap();
        let k = (j as f32 / 6.0).min(1.0);
        assert!(close(v[OPACITY], k), "opacity at {j}: {}", v[OPACITY]);
        let w = ui.tree.nodes()[i].placed.rect.width;
        assert!((w - (10.0 + 30.0 * k)).abs() <= 0.5, "width at {j}: {w}");
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}
