//! Clip playback through `og.AnimationPlayer`: markers fire in the tick
//! playback passes them (forwards, backwards, across the loop), a clip
//! change cross-fades and fires both clips' markers by weight, a seek moves
//! without firing, a one-shot clip holds its last frame and says so, and
//! the pose is the clip's own samples at sampled times.
//!
//! Failure modes covered: an off-by-one tick (firing at the tick that
//! reaches a marker instead of the one that passes it, or the tick after),
//! a marker lost or doubled at the loop seam, reverse playback ignored, a
//! fade that pops, a seek replaying markers, `time` drifting from the clip.

mod support;

use og_anim::*;
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

fn world() -> (World, AnimationSystem) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    (w, anim)
}

fn spawn(w: &mut World, player: AnimationPlayer) -> Entity {
    w.spawn((
        Transform::default(),
        Skeleton {
            asset: SKELETON,
            ..Skeleton::default()
        },
        player,
    ))
    .unwrap()
}

/// Steps `n` ticks; returns (tick the event was sent in, name, weight).
fn events(w: &mut World, n: u64) -> Vec<(u64, String, f32)> {
    let mut out = Vec::new();
    for _ in 0..n {
        let tick = w.time().tick;
        w.step().unwrap();
        for e in w.pending_events::<AnimationEvent>().unwrap() {
            out.push((tick, e.name.as_str().to_string(), e.weight));
        }
    }
    out
}

/// The tick (from 0, at 60 Hz) whose span `[k/60, (k+1)/60)` holds `t`.
fn tick_of(t: f64) -> u64 {
    (t * 60.0).floor() as u64
}

#[test]
fn markers_fire_in_the_tick_playback_passes_them() {
    let (mut w, _) = world();
    spawn(&mut w, AnimationPlayer::new(WALK));
    let got = events(&mut w, 180);
    let mut want = Vec::new();
    for cycle in 0..3 {
        for (name, t) in [("footL", 0.26), ("footR", 0.76)] {
            want.push((tick_of(cycle as f64 + t), name.to_string(), 1.0));
        }
    }
    assert_eq!(got, want);
}

#[test]
fn backwards_playback_fires_markers_on_the_way_back() {
    let (mut w, _) = world();
    spawn(
        &mut w,
        AnimationPlayer {
            speed: -1.0,
            ..AnimationPlayer::new(WALK)
        },
    );
    // From 0 backwards: the clip wraps to its end at once, then runs down.
    let got = events(&mut w, 60);
    let want = vec![
        (tick_of(1.0 - 0.76), "footR".to_string(), 1.0),
        (tick_of(1.0 - 0.26), "footL".to_string(), 1.0),
    ];
    assert_eq!(got, want);
}

#[test]
fn double_speed_fires_twice_as_often() {
    let (mut w, _) = world();
    spawn(
        &mut w,
        AnimationPlayer {
            speed: 2.0,
            ..AnimationPlayer::new(WALK)
        },
    );
    let got: Vec<u64> = events(&mut w, 60).into_iter().map(|e| e.0).collect();
    // 2 cycles per second: markers at clip time 0.26, 0.76 every half second.
    let want: Vec<u64> = [0.13, 0.38, 0.63, 0.88]
        .iter()
        .map(|&t| tick_of(t))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn a_clip_change_cross_fades_and_fires_both_clips_markers_by_weight() {
    let (mut w, anim) = world();
    let e = spawn(
        &mut w,
        AnimationPlayer {
            fade: 0.5,
            ..AnimationPlayer::new(RUN)
        },
    );
    events(&mut w, 6); // run at 0.1 s
    let mut p: AnimationPlayer = w.get(e).unwrap();
    p.clip = WALK;
    w.insert(e, p).unwrap();
    // During the fade both clips' markers fire, weighted by the fade.
    let got = events(&mut w, 30);
    let run_marker = got.iter().find(|g| g.1 == "footL").unwrap();
    // Run's footL is at 0.16 s of run: 4 ticks into the fade from 0.1 s.
    assert_eq!(run_marker.0, 6 + tick_of(0.16 - 0.1));
    let fade_at = |tick: u64| (tick - 6 + 1) as f32 / 60.0 / 0.5;
    assert!((run_marker.2 - (1.0 - fade_at(run_marker.0))).abs() < 1e-5);
    let walk_marker = got
        .iter()
        .find(|g| g.1 == "footL" && g.0 > run_marker.0)
        .unwrap();
    // Walk restarts at 0 on the change: its footL is 0.26 s after it.
    assert_eq!(walk_marker.0, 6 + tick_of(0.26));
    assert!((walk_marker.2 - fade_at(walk_marker.0)).abs() < 1e-5);
    // The pose during the fade lies between the clips, and equals walk after it.
    let mid = anim.pose(e).unwrap();
    let thigh = |pose: &[Trs]| pose[4].rotation;
    events(&mut w, 30); // past the fade: 0.5 s of walk plus one tick
    let after = anim.pose(e).unwrap();
    let mut only_walk = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim2 = install(&mut only_walk).unwrap();
    register(&anim2);
    let e2 = spawn(&mut only_walk, AnimationPlayer::new(WALK));
    for _ in 0..60 {
        only_walk.step().unwrap();
    }
    let walk_pose = anim2.pose(e2).unwrap();
    assert!(
        after.iter().zip(&walk_pose).all(|(a, b)| a == b),
        "after the fade the pose is walk's, bit for bit"
    );
    assert!(thigh(&mid) != thigh(&after));
}

#[test]
fn a_seek_moves_without_firing_and_a_one_shot_holds_its_end() {
    let (mut w, anim) = world();
    let e = spawn(
        &mut w,
        AnimationPlayer {
            looping: false,
            ..AnimationPlayer::new(WALK)
        },
    );
    events(&mut w, 1);
    let mut p: AnimationPlayer = w.get(e).unwrap();
    p.time = 0.9;
    w.insert(e, p).unwrap();
    // The seek jumps over both markers without firing them.
    let got = events(&mut w, 12);
    assert!(got.is_empty(), "{got:?}");
    let p: AnimationPlayer = w.get(e).unwrap();
    assert_eq!(p.time, 1.0);
    assert!(p.finished);
    // The held pose is the clip's last sample: hips 1.2 m forward.
    let pose = anim.pose(e).unwrap();
    assert!((pose[1].translation - Vec3::new(0.0, 1.03, 1.2)).length() < 1e-5);
}

#[test]
fn the_pose_at_a_sampled_time_is_the_clips_sample() {
    let (mut w, anim) = world();
    let e = spawn(&mut w, AnimationPlayer::new(WALK));
    let walk = anim.clip(WALK).unwrap();
    for k in 1..=20u32 {
        w.step().unwrap();
        let t = k as f32 / 60.0;
        let pose = anim.pose(e).unwrap();
        // Ticks at 60 Hz land on every second 30 Hz sample.
        if k % 2 == 0 {
            let i = (k / 2) as usize;
            for ch in &walk.channels {
                let n = ch.node as usize;
                let s = &ch.samples[i * ch.lanes as usize..][..ch.lanes as usize];
                let got: Vec<f32> = match ch.path {
                    ChannelPath::Translation => pose[n].translation.to_array().to_vec(),
                    ChannelPath::Rotation => pose[n].rotation.to_array().to_vec(),
                    _ => continue,
                };
                for (a, b) in got.iter().zip(s) {
                    assert!((a - b).abs() < 1e-6, "t {t}: {got:?} vs {s:?}");
                }
            }
        }
        assert!((w.get::<AnimationPlayer>(e).unwrap().time - t).abs() < 1e-5);
    }
}

#[test]
fn a_marker_at_zero_fires_on_the_first_tick_and_at_every_loop() {
    let (mut w, anim) = world();
    let mut c = support::clips().remove(1).1; // walk
    c.events = vec![ClipEvent {
        name: "start".into(),
        time: 0.0,
    }];
    anim.add_clip(99, c).unwrap();
    spawn(&mut w, AnimationPlayer::new(99));
    let got = events(&mut w, 150);
    let ticks: Vec<u64> = got.iter().map(|g| g.0).collect();
    assert_eq!(ticks.len(), 3, "{got:?}");
    assert_eq!(ticks[0], 0);
    // The loop seam lands within a tick of 60 and 120 (f32 time).
    assert!(
        ticks[1].abs_diff(60) <= 1 && ticks[2].abs_diff(120) <= 1,
        "{ticks:?}"
    );
}

#[test]
fn a_stepped_player_holds_each_pose_for_its_step() {
    let (mut w, anim) = world();
    let smooth = spawn(&mut w, AnimationPlayer::new(WALK));
    let held = spawn(
        &mut w,
        AnimationPlayer {
            step_fps: 12.0,
            ..AnimationPlayer::new(WALK)
        },
    );
    let walk = anim.clip(WALK).unwrap();
    let (mut last, mut changes, mut smooth_last, mut smooth_changes) = (None, 0, None, 0);
    for k in 1..=60u32 {
        w.step().unwrap();
        let p: AnimationPlayer = w.get(held).unwrap();
        // Time, and with it markers and root motion, stays continuous.
        let off = (p.time - k as f32 / 60.0).rem_euclid(1.0);
        assert!(off.min(1.0 - off) < 1e-4, "tick {k}: {}", p.time);
        let pose = anim.pose(held).unwrap();
        // Where the held time is an even twelfth it is a 30 Hz sample:
        // j/12 s is sample 5j/2.
        let j = (p.time * 12.0).floor() as usize;
        if j.is_multiple_of(2) {
            let i = j * 5 / 2;
            for ch in &walk.channels {
                let n = ch.node as usize;
                let s = &ch.samples[i * ch.lanes as usize..][..ch.lanes as usize];
                let got: Vec<f32> = match ch.path {
                    ChannelPath::Translation => pose[n].translation.to_array().to_vec(),
                    ChannelPath::Rotation => pose[n].rotation.to_array().to_vec(),
                    _ => continue,
                };
                for (a, b) in got.iter().zip(s) {
                    assert!((a - b).abs() < 1e-4, "tick {k}: {got:?} vs {s:?}");
                }
            }
        }
        changes += (last.as_ref() != Some(&pose)) as u32;
        last = Some(pose);
        let sp = anim.pose(smooth).unwrap();
        smooth_changes += (smooth_last.as_ref() != Some(&sp)) as u32;
        smooth_last = Some(sp);
    }
    // One second at 12 steps a second: 12 poses (the first counted on
    // arrival), each held for 5 ticks of 60; smooth playback moves every tick.
    assert!((12..=13).contains(&changes), "{changes} pose changes");
    assert_eq!(smooth_changes, 60);
}

#[test]
fn stepped_time_floors_to_the_step_and_ends_on_the_last_frame() {
    assert_eq!(stepped(0.49, 0.0, 1.0, true), 0.49);
    assert_eq!(stepped(0.49, 12.0, 1.0, true), 5.0 / 12.0);
    assert_eq!(stepped(0.5, 4.0, 1.0, true), 0.5);
    // A one-shot that reached its end holds its last frame, on the step or not.
    assert_eq!(stepped(0.95, 12.0, 0.95, false), 0.95);
    assert_eq!(stepped(0.95, 12.0, 0.95, true), 11.0 / 12.0);
}
