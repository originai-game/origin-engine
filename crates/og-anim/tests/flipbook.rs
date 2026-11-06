//! Flipbooks: the sheet text and the og.flipbook system, tick by tick.
//!
//! Ways this could fail: grid frames are numbered column-major or skip the
//! margin; a clip resolves a frame name to the wrong index; the frame
//! changes on the wrong tick (fps against the tick rate); ping-pong repeats
//! its end frames; a once clip wraps or never finishes; an event fires
//! twice per pass, never, or across a seek; a clip change continues from
//! the old time; time scale and hit stop are ignored; a snapshot loses the
//! playback position (events fire again, or the frame jumps).

use og_anim::*;
use og_ecs::{Entity, World, WorldDesc};

const SHEET: &str = r#"type flipbook
version 1
---
texture "sprites/hero.png"
size 66 34
margin 1
spacing 0
pivot 0.5 0
grid 16 16
frame "slash" 0 32 32 2 0.25 0.5
clip walk {
  frames 0 1 2 3
  fps 10
  event 2 "step"
}
clip wave {
  frames 4 5 6 7
  fps 10
  mode pingPong
  event 3 "top"
}
clip attack {
  frames 4 "slash"
  durations 0.05 0.1
  mode once
  event 1 "hit"
}
"#;

fn world() -> (World, FlipbookSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let fb = install_flipbook(&mut w).unwrap();
    fb.add_sheet(5, FlipbookSheet::parse(SHEET).unwrap())
        .unwrap();
    let e = w
        .spawn((Flipbook {
            sheet: 5,
            ..Default::default()
        },))
        .unwrap();
    (w, fb, e)
}

fn flip(w: &World, e: Entity) -> Flipbook {
    w.get::<Flipbook>(e).unwrap()
}

/// Steps `n` ticks; per tick, the frame shown and the events sent.
fn run(w: &mut World, e: Entity, n: u32) -> Vec<(u32, Vec<String>)> {
    (0..n)
        .map(|_| {
            w.step().unwrap();
            let ev = w
                .pending_events::<FlipbookEvent>()
                .unwrap()
                .iter()
                .map(|v| format!("{}@{}", v.name.as_str(), v.frame))
                .collect();
            (flip(w, e).frame, ev)
        })
        .collect()
}

#[test]
fn the_sheet_reads_grid_and_named_frames_and_round_trips() {
    let s = FlipbookSheet::parse(SHEET).unwrap();
    // 66 px wide with a 1 px margin: four 16 px columns, two rows.
    assert_eq!(s.frames.len(), 9);
    assert_eq!(
        s.frames[5].rect,
        [17, 17, 16, 16],
        "row-major after the margin"
    );
    assert_eq!(s.frames[8].name, "slash");
    assert_eq!(s.frames[8].pivot, [0.25, 0.5]);
    assert_eq!(s.frames[0].pivot, [0.5, 0.0]);
    assert_eq!(s.clips[2].frames, vec![4, 8]);
    assert_eq!(s.clips[0].durations, vec![0.1; 4]);
    assert_eq!(s.clip("wave"), Some(1));
    assert_eq!(s.uv(1), [17.0 / 66.0, 1.0 / 34.0, 33.0 / 66.0, 17.0 / 34.0]);
    let again = FlipbookSheet::parse(&s.to_text()).unwrap();
    assert_eq!(again, s);
    assert!(!s.array && !s.to_text().contains("array"));
    // `array true` survives the canonical text; anything else is refused.
    let arr =
        FlipbookSheet::parse(&SHEET.replace("pivot 0.5 0", "array true\npivot 0.5 0")).unwrap();
    assert!(arr.array);
    assert_eq!(FlipbookSheet::parse(&arr.to_text()).unwrap(), arr);
    for (bad, says) in [
        (
            SHEET.replace("pivot 0.5 0", "array maybe\npivot 0.5 0"),
            "array is true or false",
        ),
        (
            SHEET.replace("frames 0 1 2 3", "frames 0 1 99"),
            "no frame `99`",
        ),
        (
            SHEET.replace("mode pingPong", "mode sideways"),
            "mode is loop",
        ),
        (
            SHEET.replace("event 1 \"hit\"", "event 5 \"hit\""),
            "on frame 5 of 2",
        ),
        (
            SHEET.replace("fps 10\n  event 2", "fps 0\n  event 2"),
            "fps must be above 0",
        ),
    ] {
        let e = FlipbookSheet::parse(&bad).unwrap_err().reason().to_string();
        assert!(e.contains(says) && e.starts_with("line "), "{e}");
    }
}

#[test]
fn a_clip_change_restarts_a_seek_is_silent_and_speed_scales() {
    let (mut w, _, e) = world();
    run(&mut w, e, 20);
    let mut f = flip(&w, e);
    f.clip = 1;
    w.insert(e, f).unwrap();
    run(&mut w, e, 1);
    assert_eq!(flip(&w, e).frame, 0, "a new clip starts at its first frame");
    assert!(flip(&w, e).time < 0.02);
    // A seek within a clip lands without the events it skipped.
    let mut f = flip(&w, e);
    f.time = 0.38;
    w.insert(e, f).unwrap();
    let ticks = run(&mut w, e, 1);
    assert_eq!(
        ticks[0],
        (3, vec![]),
        "wave's top frame, without its `top` event"
    );
    // Seek walk to 0.25 s (frame 2) together with the clip change: no step event.
    let mut f = flip(&w, e);
    f.clip = 0;
    f.time = 0.25;
    w.insert(e, f).unwrap();
    let ticks = run(&mut w, e, 1);
    assert_eq!(ticks[0], (2, vec![]));
    // Twice as fast, then paused.
    let mut f = flip(&w, e);
    f.speed = 2.0;
    w.insert(e, f).unwrap();
    let before = flip(&w, e).time;
    run(&mut w, e, 6);
    // (time stays inside the 0.4 s cycle)
    assert!(
        (flip(&w, e).time - (before + 0.2).rem_euclid(0.4)).abs() < 1e-4,
        "{before} -> {}",
        flip(&w, e).time
    );
    let mut f = flip(&w, e);
    f.playing = false;
    w.insert(e, f).unwrap();
    let held = flip(&w, e);
    run(&mut w, e, 10);
    assert_eq!(flip(&w, e).time, held.time);
}

const GRAPH: &str = r#"type animgraph
version 1
---
param speed
param attack
clip stand "wave"
clip walk "walk"
clip strike "attack"
layer base {
  state idle stand
  state move walk
  state hit strike
  entry idle
  transition idle move {
    when speed gt 0.5
  }
  transition move idle {
    when speed le 0.5
  }
  transition any hit {
    when attack gt 0.5
  }
  transition hit idle {
    exit 1
  }
}
"#;

/// Steps one tick with the graph's parameters; the clip and frame shown
/// and the events sent.
fn graph_tick(w: &mut World, e: Entity, speed: f32, attack: f32) -> (u32, u32, Vec<String>) {
    let mut g = w.get::<AnimationGraph>(e).unwrap();
    (g.p0, g.p1) = (speed, attack);
    w.insert(e, g).unwrap();
    w.step().unwrap();
    let ev = w
        .pending_events::<FlipbookEvent>()
        .unwrap()
        .iter()
        .map(|v| format!("{}@{}", v.name.as_str(), v.frame))
        .collect();
    let f = flip(w, e);
    (f.clip, f.frame, ev)
}

/// An animation graph drives a flipbook by clip names: the entry state
/// shows at once, a transition cuts in only when the frame on screen
/// ends (never shortening it, never waiting for the clip's end), a once
/// clip's `exit 1` returns as soon as it finished, and a restored world
/// continues the same. Ways this could fail: the cut is immediate (wave's
/// first frame shortened), waits for the whole cycle, a fade blends
/// nothing and delays the cut, the attack's events are lost in the cut,
/// the machine is not part of the snapshot.
#[test]
fn a_graph_drives_a_flipbook_cutting_on_frame_boundaries() {
    let (mut w, fb, e) = world();
    fb.add_graph(9, std::sync::Arc::new(SpriteGraph::parse(GRAPH).unwrap()))
        .unwrap();
    w.insert(e, AnimationGraph::new(9)).unwrap();
    let wave = 1;
    let walk = 0;
    let attack = 2;
    // Entry: wave from the first tick.
    assert_eq!(graph_tick(&mut w, e, 0.0, 0.0).0, wave);
    graph_tick(&mut w, e, 0.0, 0.0);
    graph_tick(&mut w, e, 0.0, 0.0);
    // 0.05 s into wave's 0.1 s first frame: the walk waits for it to end.
    let mut seq = Vec::new();
    for _ in 0..6 {
        seq.push(graph_tick(&mut w, e, 1.0, 0.0));
    }
    let cut = seq.iter().position(|t| t.0 == walk).expect("walk cut in");
    assert!(
        seq[..cut].iter().all(|t| (t.0, t.1) == (wave, 0)),
        "wave's frame 0 until the cut: {seq:?}"
    );
    // Six ticks of 1/60 s hold the 0.1 s frame: the cut is the 6th tick.
    assert_eq!(cut + 3 + 1, 6, "{seq:?}");
    assert_eq!(seq[cut].1, 0, "walk from its first frame");
    assert_eq!(w.get::<AnimationGraph>(e).unwrap().state, 1, "move");
    // Attack: stop and strike; the strike cuts in at walk's frame end,
    // fires its hit and returns to idle (wave) as soon as it finished.
    let snap = w.snapshot();
    let mut a = Vec::new();
    for i in 0..30 {
        a.push(graph_tick(&mut w, e, 0.0, if i < 3 { 1.0 } else { 0.0 }));
    }
    let start = a.iter().position(|t| t.0 == attack).expect("the strike");
    let back = a[start..]
        .iter()
        .position(|t| t.0 == wave)
        .expect("back to wave")
        + start;
    assert!(a.iter().flat_map(|t| &t.2).any(|ev| ev == "hit@1"), "{a:?}");
    // 0.15 s of strike: 9 ticks, the 10th returns.
    assert_eq!(back - start, 9, "{a:?}");
    // A restored world plays the same ticks.
    w.restore(&snap).unwrap();
    let mut b = Vec::new();
    for i in 0..30 {
        b.push(graph_tick(&mut w, e, 0.0, if i < 3 { 1.0 } else { 0.0 }));
    }
    assert_eq!(a, b);
}
