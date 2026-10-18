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
    for (bad, says) in [
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
fn a_looping_clip_advances_at_its_rate_and_fires_once_per_pass() {
    let (mut w, _, e) = world();
    let ticks = run(&mut w, e, 48);
    // 10 fps at 60 Hz: six ticks a frame.
    let frames: Vec<u32> = ticks.iter().map(|t| t.0).collect();
    let mut expected = Vec::new();
    for i in 0..48 {
        expected.push(((i + 1) / 6 % 4) as u32);
    }
    assert_eq!(frames, expected);
    let steps: Vec<usize> = ticks
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.1.is_empty())
        .map(|(i, _)| i)
        .collect();
    assert_eq!(steps, vec![11, 35], "frame 2 entered at ticks 12 and 36");
    assert_eq!(flip(&w, e).loops, 2);
}

#[test]
fn ping_pong_turns_at_the_ends_and_once_holds_and_finishes() {
    let (mut w, _, e) = world();
    w.insert(
        e,
        Flipbook {
            sheet: 5,
            clip: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let ticks = run(&mut w, e, 6 * 7);
    let order: Vec<u32> = ticks.iter().step_by(6).map(|t| t.0).collect();
    assert_eq!(order, vec![0, 1, 2, 3, 2, 1, 0]);
    assert_eq!(
        ticks.iter().filter(|t| !t.1.is_empty()).count(),
        1,
        "the top is passed once per cycle"
    );

    w.insert(
        e,
        Flipbook {
            sheet: 5,
            clip: 2,
            ..Default::default()
        },
    )
    .unwrap();
    let ticks = run(&mut w, e, 20);
    assert_eq!(ticks[1].0, 0);
    assert_eq!(ticks[2].0, 1, "0.05 s is three ticks");
    assert!(flip(&w, e).finished);
    assert_eq!(flip(&w, e).frame, 1, "held on the last frame");
    let hits: Vec<&Vec<String>> = ticks
        .iter()
        .map(|t| &t.1)
        .filter(|v| !v.is_empty())
        .collect();
    assert_eq!(hits, vec![&vec!["hit@1".to_string()]]);
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

#[test]
fn slow_motion_and_hit_stop_slow_and_hold_the_frame() {
    let (mut w, _, e) = world();
    w.set_time_scale(0.5).unwrap();
    let ticks = run(&mut w, e, 12);
    assert_eq!(ticks[10].0, 0);
    assert_eq!(ticks[11].0, 1, "twelve ticks a frame at half speed");
    w.hit_stop(30).unwrap();
    let t = flip(&w, e).time;
    run(&mut w, e, 30);
    assert_eq!(flip(&w, e).time, t);
}

#[test]
fn a_snapshot_keeps_the_playback_and_the_transcript_is_the_same() {
    let (mut w, _, e) = world();
    let first = run(&mut w, e, 25);
    let snap = w.snapshot();
    let rest = run(&mut w, e, 60);
    let (mut r, _, _) = world();
    r.restore(&snap).unwrap();
    let again = run(&mut r, e, 60);
    assert_eq!(again, rest);
    assert_eq!(r.world_hash(), w.world_hash());
    // The transcript of the whole run, for tests/evidence/E4.
    let all: Vec<_> = first
        .iter()
        .chain(&rest)
        .enumerate()
        .filter(|(_, t)| !t.1.is_empty())
        .map(|(i, t)| {
            format!(
                "{{\"tick\":{},\"frame\":{},\"events\":{:?}}}",
                i + 1,
                t.0,
                t.1
            )
        })
        .collect();
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("flipbook-events.json");
    std::fs::write(&path, format!("[\n{}\n]\n", all.join(",\n"))).unwrap();
}
