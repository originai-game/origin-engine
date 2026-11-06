//! The E6 mixer and tick features, through the public boundary: a world
//! with `og_audio::install`, its command stream rendered by a capture sink.
//!
//! Ways they could fail, and the test that catches each:
//! - a bus insert is not applied, applied to the wrong bus, or its cutoff is
//!   ignored → `bus_filters_shape_their_bus_only`
//! - ducking never engages, never releases, or keys off the wrong bus →
//!   `ducking_turns_music_down_under_effects_and_back`
//! - the compressor does not reduce a loud bus → `the_compressor_reduces_a_loud_bus`
//! - the reverb send leaves no tail, or plays without a send →
//!   `a_reverb_send_leaves_a_tail`
//! - a scheduled start lands on the tick instead of its sample, or two stems
//!   started together drift → `scheduled_starts_land_on_their_sample_and_stems_stay_aligned`
//! - the music clock's beats, bars or next-bar times are off →
//!   `the_music_clock_beats_and_a_stinger_lands_on_the_bar_line`
//! - a crossfade cuts instead of ramping, or ignores its length →
//!   `a_crossfade_ramps_over_its_fade`
//! - the voice cap lets too many play, culls the wrong one, or stops a
//!   culled source silently → `the_voice_cap_culls_the_lowest_priority`
//! - occlusion or distance does not dull the sound →
//!   `occlusion_and_distance_dull_a_source`
//! - the mixer's output depends on anything but the stream, or `listen`
//!   returns other frames than the ones rendered →
//!   `the_mix_is_the_same_twice_and_listen_returns_its_last_second`
//! - a source with a startOffset starts from the clip's beginning, a frame
//!   off, fades in over the first samples, loses its startAt inside the
//!   tick, or ends on another tick than the rest of the clip does →
//!   `a_start_part_way_plays_the_clip_from_its_offset_to_the_sample`
//! - a reverb zone does not reach the stream, ignores where the listener
//!   is, jumps at its edge instead of fading, leaves the open air changed
//!   behind it, or eases differently on a second run or after a restore →
//!   `a_reverb_zone_eases_the_shared_reverb_by_where_the_listener_is`

use crate::support;

use og_audio::command::fx;
use og_audio::{
    Audio, AudioBusFx, AudioDesc, AudioLink, AudioListener, AudioSource, AudioStream, Bus,
    HeadlessMode, HeadlessSink, Op, ReverbZone, SoundBank, Transform, install,
};
use og_core::glam::Vec3;
use og_ecs::{Entity, World, WorldDesc};
use std::f64::consts::TAU;
use std::sync::Arc;
use support::*;

fn step(world: &mut World, sink: &mut HeadlessSink, ticks: u64) {
    for _ in 0..ticks {
        world.step().unwrap();
        sink.pump();
    }
}

/// Goertzel magnitude of `freq` in a channel, as an amplitude.
fn tone(ch: &[f32], freq: f64) -> f64 {
    let n = ch.len() as f64;
    let w = TAU * freq / f64::from(RATE);
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &x in ch {
        let s0 = f64::from(x) + c * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - c * s1 * s2).max(0.0).sqrt() * 2.0 / n
}

/// Frames `a..b` of the left channel.
fn window(pcm: &[f32], a: usize, b: usize) -> Vec<f32> {
    pcm[a * 2..b * 2].iter().step_by(2).copied().collect()
}

fn looped(sound: u64, bus: Bus, volume: f32) -> AudioSource {
    AudioSource {
        looping: true,
        bus: bus as u32,
        volume,
        ..AudioSource::new(sound)
    }
}

fn capture(world: &mut World, link: &AudioLink) -> HeadlessSink {
    let _ = world;
    HeadlessSink::new(link, HeadlessMode::Capture { sample_rate: RATE })
}

/// The shared reverb as the last fx commands set it: (room, damping, the
/// sfx bus's send), or `None` when no packet carried any.
fn reverb_sent(link: &AudioLink) -> Option<(f32, f32, f32)> {
    let mut out = None;
    for packet in link.drain() {
        let mut got = out.unwrap_or((-1.0, -1.0, -1.0));
        let mut any = false;
        for c in packet.commands.iter().filter(|c| c.op == Op::Fx) {
            any = true;
            match (c.bus, c.voice) {
                (0, fx::ROOM) => got.0 = c.gain,
                (0, fx::DAMPING) => got.1 = c.gain,
                (2, fx::REVERB) => got.2 = c.gain,
                _ => {}
            }
        }
        if any {
            out = Some(got);
        }
    }
    out
}

/// kestrel-stair #21: an og.ReverbZone eases the shared reverb (room,
/// damping, a bus's send) towards its own while the listener is inside,
/// fading over `fade` metres past its edge, as a function of the
/// listener's position each tick.
#[test]
fn a_reverb_zone_eases_the_shared_reverb_by_where_the_listener_is() {
    let bank = SoundBank::new();
    let click = insert(&bank, "click", 1, &synth(480, 1, sine(1000.0, 0.8)));
    let setup = || {
        let (mut world, link) = world_with(&bank, |w| {
            w.register::<AudioListener>().unwrap();
        });
        *world.resource_mut::<Audio>().unwrap() = Audio {
            reverb_room: 0.5,
            reverb_damping: 0.5,
            ..Audio::default()
        };
        world
            .spawn((AudioBusFx {
                bus: Bus::Sfx as u32,
                reverb: 0.0,
                ..AudioBusFx::default()
            },))
            .unwrap();
        let ear = world
            .spawn((Transform::default(), AudioListener::default()))
            .unwrap();
        // A crypt: a 10 x 4 x 10 m box 20 m ahead, turned a quarter.
        let crypt = world
            .spawn((
                Transform {
                    position: Vec3::new(0.0, 0.0, -20.0),
                    quaternion: og_core::glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Transform::default()
                },
                ReverbZone {
                    size: Vec3::new(10.0, 4.0, 10.0),
                    fade: 4.0,
                    send: 0.6,
                    room: 0.9,
                    damping: 0.2,
                    ..ReverbZone::default()
                },
            ))
            .unwrap();
        link.attach();
        (world, link, ear, crypt)
    };
    let at = |world: &mut World, ear: Entity, z: f32| {
        world
            .insert(
                ear,
                Transform {
                    position: Vec3::new(0.0, 1.0, z),
                    ..Transform::default()
                },
            )
            .unwrap();
        world.step().unwrap();
    };
    let near = |a: (f32, f32, f32), b: (f32, f32, f32)| {
        (a.0 - b.0).abs() < 1e-5 && (a.1 - b.1).abs() < 1e-5 && (a.2 - b.2).abs() < 1e-5
    };
    let (mut world, link, ear, crypt) = setup();
    // Far outside: the open air, untouched.
    at(&mut world, ear, 0.0);
    let open = reverb_sent(&link).expect("the first tick sends every parameter");
    assert!(near(open, (0.5, 0.5, 0.0)), "open air {open:?}");
    assert_eq!(world.get::<ReverbZone>(crypt).unwrap().weight, 0.0);
    // Two metres from its edge, half of a 4 m fade: halfway.
    at(&mut world, ear, -13.0);
    let half = reverb_sent(&link).expect("the zone changed the reverb");
    assert!(
        near(half, (0.7, 0.35, 0.3)),
        "at the fade's middle {half:?}"
    );
    let w = world.get::<ReverbZone>(crypt).unwrap().weight;
    assert!((w - 0.5).abs() < 1e-5, "weight {w}");
    // Inside: the crypt's own.
    at(&mut world, ear, -18.0);
    let inside = reverb_sent(&link).expect("deeper in, the zone's own");
    assert!(near(inside, (0.9, 0.2, 0.6)), "inside {inside:?}");
    assert_eq!(world.get::<ReverbZone>(crypt).unwrap().weight, 1.0);
    // Standing still sends nothing more.
    at(&mut world, ear, -18.0);
    assert_eq!(reverb_sent(&link), None, "no change, no commands");
    // Out again: the open air comes back exactly.
    at(&mut world, ear, 0.0);
    let back = reverb_sent(&link).expect("leaving restores the open air");
    assert!(near(back, open), "back outside {back:?}");
    // The open air's own settings were never written.
    let audio = *world.resource::<Audio>().unwrap();
    assert_eq!((audio.reverb_room, audio.reverb_damping), (0.5, 0.5));

    // The same walk twice is the same stream; a restore mid-walk goes on
    // as the walk did.
    let walk = |restore_at: Option<usize>| {
        let (mut world, _link, ear, _) = setup();
        let mut saved = None;
        for (i, z) in (0..120).map(|i| (i, -(i as f32) * 0.25)) {
            if Some(i) == restore_at {
                saved = Some(world.snapshot());
            }
            at(&mut world, ear, z);
        }
        let end = (
            world.resource::<AudioStream>().unwrap().hash,
            world.world_hash(),
        );
        if let Some(bytes) = saved {
            world.restore(&bytes).unwrap();
            for z in (restore_at.unwrap()..120).map(|i| -(i as f32) * 0.25) {
                at(&mut world, ear, z);
            }
            assert_eq!(
                (
                    world.resource::<AudioStream>().unwrap().hash,
                    world.world_hash()
                ),
                end,
                "restored mid-walk"
            );
        }
        end
    };
    assert_eq!(walk(None), walk(Some(55)));

    // It reaches the mix: a click inside the zone rings, outside it does not.
    let tail = |z: f32| {
        let (mut world, link, ear, _) = setup();
        link.detach();
        at(&mut world, ear, z);
        world.spawn((AudioSource::new(click),)).unwrap();
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 60);
        let pcm = sink.pcm().to_vec();
        rms(&window(&pcm, 9_600, 24_000))
    };
    let (dry, wet) = (tail(0.0), tail(-20.0));
    assert!(dry < 1e-4, "outside, no send: {dry}");
    assert!(wet > 3e-3, "inside the crypt the click rings: {wet}");
}

#[test]
fn occlusion_and_distance_dull_a_source() {
    let bank = SoundBank::new();
    let noise = synth(RATE as usize, 1, |i, _| {
        // A hash of the frame index: white noise, the same every run.
        let mut x = (i as u32).wrapping_mul(0x9e37_79b9) ^ 0x85eb_ca6b;
        x ^= x >> 15;
        x = x.wrapping_mul(0x2c1b_3c6d);
        x ^= x >> 12;
        (f64::from(x) / f64::from(u32::MAX) - 0.5) * 0.6
    });
    let n = insert(&bank, "n", 1, &noise);
    // (high band over low band, the low band's level) of a spatial source.
    let measure = |occluded: bool, distance: f32, cutoff: f32| {
        let mut world = World::new(&WorldDesc {
            tick_rate: 60,
            seed: 7,
        })
        .unwrap();
        world.register::<Transform>().unwrap();
        let link = install(
            &mut world,
            &AudioDesc {
                bank: Some(bank.clone()),
                occluder: Some(Arc::new(move |_, _| occluded)),
                ..AudioDesc::default()
            },
        )
        .unwrap();
        world
            .spawn((
                Transform::at(Vec3::new(0.0, 0.0, -distance)),
                AudioSource {
                    spatial: true,
                    occlude: true,
                    distance_cutoff: cutoff,
                    ref_distance: 1.0,
                    max_distance: 50.0,
                    rolloff_factor: 0.0,
                    ..looped(n, Bus::Sfx, 1.0)
                },
            ))
            .unwrap();
        let mut sink = capture(&mut world, &link);
        step(&mut world, &mut sink, 30);
        let w = window(sink.pcm(), 12_000, 24_000);
        let hi: f64 = [4000.0, 6000.0, 8000.0, 10000.0]
            .iter()
            .map(|&f| tone(&w, f))
            .sum();
        let lo: f64 = [200.0, 300.0, 400.0, 500.0]
            .iter()
            .map(|&f| tone(&w, f))
            .sum();
        (hi / lo, lo)
    };
    let (clear, clear_level) = measure(false, 2.0, 0.0);
    let (behind, behind_level) = measure(true, 2.0, 0.0);
    let (far, _) = measure(false, 49.0, 1000.0);
    let (near, _) = measure(false, 1.0, 1000.0);
    assert!(
        behind < clear * 0.3,
        "occlusion takes the highs: {behind:.3} vs {clear:.3}"
    );
    let lows = behind_level / clear_level;
    assert!(
        lows > 0.25 && lows < 0.4,
        "and the level, to occlusionGain 0.35 below its cutoff: {lows:.3}"
    );
    assert!(
        far < near * 0.3,
        "distance takes the highs: {far:.3} vs {near:.3}"
    );
    assert!(
        (near / clear - 1.0).abs() < 0.2,
        "at refDistance nothing is filtered: {near} vs {clear}"
    );
}
