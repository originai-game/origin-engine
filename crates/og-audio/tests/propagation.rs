//! Propagation: Doppler, cone directivity and the HRTF panner, through the
//! public boundary (a world with `og_audio::install`, its stream read from
//! the link or rendered by a capture sink).
//!
//! Ways it could fail, and the test that catches each:
//! - Doppler is ignored, has the wrong sign, ignores the listener's motion
//!   or `dopplerFactor` (the source's or the listener's), or a teleport
//!   shrieks → `doppler_shifts_by_the_motion_along_the_line_between_source_and_listener`
//! - the cone is ignored, faces the wrong way, or interpolates otherwise
//!   than Web Audio's PannerNode → `a_cone_turns_a_source_down_away_from_its_front`
//! - the HRTF does not delay or shadow the far ear, a centred source is
//!   not centred, the stereo path changes when it is off, or turning it off
//!   leaves the voice on it → `the_hrtf_delays_and_shadows_the_far_ear`
//! - a feature's state is left out of the world (a velocity from outside
//!   the snapshot, a direction the mixer keeps alone), so a second run or a
//!   restore mid-walk differs →
//!   `a_walk_with_doppler_cones_and_the_hrtf_is_the_same_twice_and_across_a_restore`

use crate::support;

use og_audio::command::Op;
use og_audio::schema::panning_model;
use og_audio::{
    AudioLink, AudioListener, AudioPlayback, AudioSource, AudioStream, HeadlessMode, HeadlessSink,
    SoundBank, Transform,
};
use og_core::glam::{Quat, Vec3};
use og_core::math::quat_from_axis_angle;
use og_ecs::{Entity, World};
use support::*;

const C: f64 = 343.3;

fn listener_world(bank: &SoundBank) -> (World, AudioLink) {
    world_with(bank, |w| {
        w.register::<AudioListener>().unwrap();
    })
}

fn tone(bank: &SoundBank) -> u64 {
    insert(bank, "tone", 1, &synth(RATE as usize, 1, sine(440.0, 0.5)))
}

fn looping_spatial(sound: u64) -> AudioSource {
    AudioSource {
        looping: true,
        spatial: true,
        ref_distance: 2.0,
        ..AudioSource::new(sound)
    }
}

/// The pitch og.audio sent for `e` this tick.
fn pitch(world: &World, e: Entity) -> f32 {
    world.get::<AudioPlayback>(e).unwrap().pitch
}

/// Frequency of a signal by its zero crossings (rising) per second.
fn frequency(ch: &[f32]) -> f64 {
    let rising = ch.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
    rising as f64 * f64::from(RATE) / ch.len() as f64
}

/// OpenAL's Doppler ratio in f64, from positions and velocities.
fn expected_ratio(src: [f64; 3], vs: [f64; 3], ear: [f64; 3], vl: [f64; 3]) -> f64 {
    let d = [ear[0] - src[0], ear[1] - src[1], ear[2] - src[2]];
    let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let u = [d[0] / n, d[1] / n, d[2] / n];
    let dot = |a: [f64; 3]| a[0] * u[0] + a[1] * u[1] + a[2] * u[2];
    (C - dot(vl)) / (C - dot(vs))
}

#[test]
fn doppler_shifts_by_the_motion_along_the_line_between_source_and_listener() {
    let bank = SoundBank::new();
    let sound = tone(&bank);
    // A source 2 m to the right of the listener's line, driving along +Z
    // at 20 m/s (1/3 m a tick) from 40 m ahead to 40 m behind; 240 ticks.
    let z_at = |t: u64| -40.0 + t as f32 / 3.0;
    let drive = |source: AudioSource, listener: AudioListener, teleport: Option<u64>| {
        let (mut world, link) = listener_world(&bank);
        world.spawn((Transform::default(), listener)).unwrap();
        let e = world
            .spawn((Transform::at(Vec3::new(2.0, 0.0, z_at(0))), source))
            .unwrap();
        let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
        let mut pitches = Vec::new();
        for t in 0..240u64 {
            let mut z = z_at(t);
            if teleport.is_some_and(|at| t >= at) {
                z += 200.0;
            }
            world
                .insert(e, Transform::at(Vec3::new(2.0, 0.0, z)))
                .unwrap();
            world.step().unwrap();
            sink.pump();
            pitches.push(pitch(&world, e));
        }
        (pitches, sink.pcm().to_vec())
    };
    let (pitches, pcm) = drive(looping_spatial(sound), AudioListener::default(), None);
    assert_eq!(pitches[0], 1.0, "a voice's first tick has no velocity");
    for t in [10u64, 100, 140, 230] {
        let z = f64::from(z_at(t));
        let want = expected_ratio([2.0, 0.0, z], [0.0, 0.0, 20.0], [0.0; 3], [0.0; 3]);
        let got = f64::from(pitches[t as usize]);
        assert!(
            (got - want).abs() < 2e-4,
            "tick {t} (z {z:.2}): pitch {got}, OpenAL's model gives {want}"
        );
    }
    assert!(
        pitches[10] > 1.055,
        "approaching is higher: {}",
        pitches[10]
    );
    assert!(pitches[230] < 0.95, "receding is lower: {}", pitches[230]);
    // The shift is heard: 440 Hz sounds about 467 Hz coming, 416 Hz going.
    let (l, r) = split(&pcm);
    let mono: Vec<f32> = l.iter().zip(&r).map(|(a, b)| a + b).collect();
    let coming = frequency(&mono[5 * 800..45 * 800]);
    let going = frequency(&mono[195 * 800..235 * 800]);
    assert!(
        coming > 460.0 && coming < 472.0,
        "approaching at 20 m/s: {coming:.1} Hz"
    );
    assert!(
        going > 410.0 && going < 422.0,
        "receding at 20 m/s: {going:.1} Hz"
    );

    // dopplerFactor 0 on the source, or on the listener: no shift at all.
    let (off, _) = drive(
        AudioSource {
            doppler_factor: 0.0,
            ..looping_spatial(sound)
        },
        AudioListener::default(),
        None,
    );
    assert!(off.iter().all(|p| *p == 1.0), "source factor 0: {off:?}");
    let (deaf, _) = drive(
        looping_spatial(sound),
        AudioListener {
            doppler_factor: 0.0,
            ..AudioListener::default()
        },
        None,
    );
    assert!(deaf.iter().all(|p| *p == 1.0), "listener factor 0");
    // A factor of 2 doubles the speed it hears.
    let (double, _) = drive(
        AudioSource {
            doppler_factor: 2.0,
            ..looping_spatial(sound)
        },
        AudioListener::default(),
        None,
    );
    let z = f64::from(z_at(10));
    let want = expected_ratio([2.0, 0.0, z], [0.0, 0.0, 40.0], [0.0; 3], [0.0; 3]);
    assert!((f64::from(double[10]) - want).abs() < 2e-4, "factor 2");

    // A jump of 200 m in a tick is a teleport, not 12 km/s: no shift that
    // tick, and the drive's own shift after it.
    let (warped, _) = drive(looping_spatial(sound), AudioListener::default(), Some(70));
    assert_eq!(warped[70], 1.0, "the teleport's tick");
    let z = f64::from(z_at(71)) + 200.0;
    let want = expected_ratio([2.0, 0.0, z], [0.0, 0.0, 20.0], [0.0; 3], [0.0; 3]);
    assert!(
        (f64::from(warped[71]) - want).abs() < 2e-4,
        "the tick after: receding at 20 m/s, {} (want {want})",
        warped[71]
    );

    // The listener's own motion: walking at 5 m/s towards a still source.
    let (mut world, _link) = listener_world(&bank);
    let ear = world
        .spawn((Transform::default(), AudioListener::default()))
        .unwrap();
    let e = world
        .spawn((
            Transform::at(Vec3::new(0.0, 0.0, -50.0)),
            looping_spatial(sound),
        ))
        .unwrap();
    let mut heard = Vec::new();
    for t in 0..30u64 {
        world
            .insert(ear, Transform::at(Vec3::new(0.0, 0.0, -(t as f32) / 12.0)))
            .unwrap();
        world.step().unwrap();
        heard.push(pitch(&world, e));
    }
    let want = (C + 5.0) / C;
    assert!(
        (f64::from(heard[20]) - want).abs() < 2e-4,
        "a listener walking at 5 m/s towards it: {} (want {want})",
        heard[20]
    );
}

#[test]
fn a_cone_turns_a_source_down_away_from_its_front() {
    let bank = SoundBank::new();
    let sound = tone(&bank);
    // A source at the origin facing +Z (three.js PositionalAudio's front);
    // the listener 1 m away (inside refDistance 2: no distance loss) at an
    // angle from that front, one angle a tick.
    let angles = [0.0f32, 20.0, 45.0, 60.0, 75.0, 90.0, 150.0, 180.0];
    let gains = |source: AudioSource, facing: Quat| {
        let (mut world, _link) = listener_world(&bank);
        let ear = world
            .spawn((Transform::at(Vec3::Z), AudioListener::default()))
            .unwrap();
        let e = world
            .spawn((
                Transform {
                    quaternion: facing,
                    ..Transform::default()
                },
                AudioSource {
                    doppler_factor: 0.0,
                    ..source
                },
            ))
            .unwrap();
        angles
            .iter()
            .map(|a| {
                let r = a.to_radians();
                world
                    .insert(ear, Transform::at(Vec3::new(r.sin(), 0.0, r.cos())))
                    .unwrap();
                world.step().unwrap();
                world.get::<AudioPlayback>(e).unwrap().gain
            })
            .collect::<Vec<f32>>()
    };
    let coned = AudioSource {
        cone_inner_angle: 60.0,
        cone_outer_angle: 180.0,
        cone_outer_gain: 0.2,
        ..looping_spatial(sound)
    };
    // Web Audio: 1 within 30°, 0.2 past 90°, linear in the angle between.
    let want = [1.0f32, 1.0, 0.8, 0.6, 0.4, 0.2, 0.2, 0.2];
    let got = gains(coned, Quat::IDENTITY);
    for ((a, g), w) in angles.iter().zip(&got).zip(&want) {
        assert!((g - w).abs() < 1e-4, "{a}°: gain {g}, want {w}");
    }
    // Turned to face -Z, the listener in front of +Z is behind it.
    let turned = gains(coned, quat_from_axis_angle(Vec3::Y, std::f32::consts::PI));
    assert!((turned[0] - 0.2).abs() < 1e-4, "behind: {}", turned[0]);
    assert!((turned[7] - 1.0).abs() < 1e-4, "in front: {}", turned[7]);
    // The default (360°, 360°) is omnidirectional: full gain all round.
    assert!(
        gains(looping_spatial(sound), Quat::IDENTITY)
            .iter()
            .all(|g| *g == 1.0)
    );
}

/// White noise, the same every run (a hash of the frame index).
fn noise(frames: usize) -> Vec<i16> {
    synth(frames, 1, |i, _| {
        let mut x = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        x ^= x >> 29;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 32;
        0.4 * ((x & 0xffff) as f64 / 32768.0 - 1.0)
    })
}

/// The lag (frames) of `b` against `a` that correlates best, in ±range.
fn best_lag(a: &[f32], b: &[f32], range: i64) -> i64 {
    let n = a.len() as i64;
    (-range..=range)
        .max_by(|&x, &y| {
            let c = |k: i64| -> f64 {
                (range..n - range)
                    .map(|i| f64::from(a[i as usize]) * f64::from(b[(i + k) as usize]))
                    .sum()
            };
            c(x).total_cmp(&c(y))
        })
        .unwrap()
}

/// RMS of a channel through a one-pole low-pass at about 200 Hz: its lows.
fn low(ch: &[f32]) -> f32 {
    let a = 1.0 - (-std::f32::consts::TAU * 200.0 / RATE as f32).exp();
    let mut z = 0.0;
    let y: Vec<f32> = ch
        .iter()
        .map(|x| {
            z += a * (x - z);
            z
        })
        .collect();
    rms(&y)
}

/// RMS of a channel's first difference: its high frequencies.
fn bright(ch: &[f32]) -> f32 {
    let d: Vec<f32> = ch.windows(2).map(|w| w[1] - w[0]).collect();
    rms(&d)
}

#[test]
fn the_hrtf_delays_and_shadows_the_far_ear() {
    let bank = SoundBank::new();
    let hiss = insert(&bank, "noise", 1, &noise(RATE as usize));
    // `model` for the listener, the source at `at`, 30 ticks; the listener
    // switches to `then` from tick 15 when given.
    let hear = |model: u32, at: Vec3, then: Option<u32>| {
        let (mut world, link) = listener_world(&bank);
        let ear = world
            .spawn((
                Transform::default(),
                AudioListener {
                    panning_model: model,
                    ..AudioListener::default()
                },
            ))
            .unwrap();
        world
            .spawn((
                Transform::at(at),
                AudioSource {
                    ref_distance: 3.0,
                    ..looping_spatial(hiss)
                },
            ))
            .unwrap();
        let mut mixer = og_audio::Mixer::for_link(&link, RATE, og_audio::Clock::Driven);
        link.attach();
        let (mut pcm, mut hrtf_commands) = (Vec::new(), 0);
        for t in 0..30u64 {
            if t == 15
                && let Some(m) = then
            {
                world
                    .insert(
                        ear,
                        AudioListener {
                            panning_model: m,
                            ..AudioListener::default()
                        },
                    )
                    .unwrap();
            }
            world.step().unwrap();
            for p in link.drain() {
                hrtf_commands += p.commands.iter().filter(|c| c.op == Op::Hrtf).count();
                mixer.push(p);
            }
            mixer.render_until_tick(link.next_tick(), usize::MAX, &mut pcm);
        }
        (pcm, hrtf_commands)
    };
    let (right, sent) = hear(panning_model::HRTF, Vec3::new(3.0, 0.0, 0.0), None);
    assert_eq!(sent, 1, "one direction, sent with the play");
    let (l, r) = split(&right[2 * 800 * 2..14 * 800 * 2]);
    // The far ear hears it (a/c)(1 + π/2) later: 31.5 frames at 48 kHz.
    let itd = 0.0875 / 343.0 * (1.0 + std::f64::consts::FRAC_PI_2) * f64::from(RATE);
    let lag = best_lag(&r, &l, 48);
    assert!(
        (lag as f64 - itd).abs() <= 1.0,
        "the left ear lags by {lag} frames (Woodworth: {itd:.1})"
    );
    // And the head shadows it: the far ear's highs are well down, its lows not.
    let (hl, hr) = (bright(&l), bright(&r));
    assert!(hr > 5.0 * hl, "highs: right {hr}, left {hl}");
    let (ll, lr) = (low(&l), low(&r));
    assert!(ll > 0.7 * lr, "lows reach both ears: left {ll}, right {lr}");

    // Ahead, both ears hear the same, sample for sample.
    let (ahead, _) = hear(panning_model::HRTF, Vec3::new(0.0, 0.0, -3.0), None);
    let (al, ar) = split(&ahead);
    assert!(rms(&al) > 0.05);
    assert_eq!(al, ar, "a source ahead is centred");

    // The stereo panner sends no HRTF commands and puts a hard-right
    // source on the right only.
    let (pan, none) = hear(panning_model::EQUALPOWER, Vec3::new(3.0, 0.0, 0.0), None);
    assert_eq!(none, 0, "no hrtf commands without the HRTF");
    let (pl, pr) = split(&pan[2 * 800 * 2..14 * 800 * 2]);
    assert!(rms(&pl) < 1e-3 * rms(&pr), "equal power: hard right");

    // Turned off mid-run, the voice goes back to the panner.
    let (switched, sent) = hear(
        panning_model::HRTF,
        Vec3::new(3.0, 0.0, 0.0),
        Some(panning_model::EQUALPOWER),
    );
    assert_eq!(sent, 2, "the play's direction, then the zero vector");
    let (sl, sr) = split(&switched[17 * 800 * 2..29 * 800 * 2]);
    assert!(rms(&sl) < 1e-3 * rms(&sr), "back on the panner");
}

/// A listener walks a path with the HRTF while three spatial sources move
/// and one turns its cone; returns the PCM hash, the stream hash and the
/// world hash at the end, restoring the snapshot taken at `restore_at` on
/// the way when given.
fn walk(restore_at: Option<u64>) -> (u64, u64, u64) {
    let bank = SoundBank::new();
    let sound = tone(&bank);
    let hiss = insert(&bank, "noise", 1, &noise(RATE as usize / 2));
    let (mut world, link) = listener_world(&bank);
    let ear = world
        .spawn((
            Transform::default(),
            AudioListener {
                panning_model: panning_model::HRTF,
                ..AudioListener::default()
            },
        ))
        .unwrap();
    let movers: Vec<Entity> = (0..3)
        .map(|k| {
            world
                .spawn((
                    Transform::at(Vec3::new(k as f32 * 4.0 - 4.0, 0.0, -6.0)),
                    AudioSource {
                        cone_inner_angle: 90.0,
                        cone_outer_angle: 240.0,
                        cone_outer_gain: 0.1,
                        doppler_factor: 1.0 + k as f32 * 0.5,
                        pitch: 0.9 + 0.1 * k as f32,
                        ..looping_spatial(if k == 1 { hiss } else { sound })
                    },
                ))
                .unwrap()
        })
        .collect();
    let mut sink = HeadlessSink::new(&link, HeadlessMode::Capture { sample_rate: RATE });
    let place = |world: &mut World, t: u64| {
        let s = t as f32 / 60.0;
        world
            .insert(
                ear,
                Transform {
                    position: Vec3::new(3.0 * s.sin(), 0.0, -2.0 * s),
                    quaternion: quat_from_axis_angle(Vec3::Y, 0.4 * s),
                    scale: Vec3::ONE,
                },
            )
            .unwrap();
        for (k, &e) in movers.iter().enumerate() {
            let a = s * (1.0 + k as f32) + k as f32;
            world
                .insert(
                    e,
                    Transform {
                        position: Vec3::new(6.0 * a.cos(), 0.5 * k as f32, -6.0 + 6.0 * a.sin()),
                        quaternion: quat_from_axis_angle(Vec3::Y, a * 2.0),
                        scale: Vec3::ONE,
                    },
                )
                .unwrap();
        }
    };
    let mut saved = None;
    for t in 0..240u64 {
        if Some(t) == restore_at {
            saved = Some(world.snapshot());
        }
        place(&mut world, t);
        world.step().unwrap();
        sink.pump();
    }
    let pcm = sink.pcm().to_vec();
    let mut h = og_core::hash::StableHasher::new();
    for x in &pcm {
        h.write_u32(x.to_bits());
    }
    let end = (
        h.finish(),
        world.resource::<AudioStream>().unwrap().hash,
        world.world_hash(),
    );
    if let (Some(bytes), Some(at)) = (saved, restore_at) {
        world.restore(&bytes).unwrap();
        for t in at..240 {
            place(&mut world, t);
            world.step().unwrap();
        }
        assert_eq!(
            (
                world.resource::<AudioStream>().unwrap().hash,
                world.world_hash()
            ),
            (end.1, end.2),
            "restored at tick {at}, the walk goes on as it did"
        );
    }
    end
}

#[test]
fn a_walk_with_doppler_cones_and_the_hrtf_is_the_same_twice_and_across_a_restore() {
    let a = walk(None);
    assert_eq!(
        walk(None),
        a,
        "the same walk twice: the same samples and stream"
    );
    assert_eq!(walk(Some(117)), a);
}
