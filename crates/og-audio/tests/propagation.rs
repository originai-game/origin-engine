//! Audio end to end: a world with `og_audio::install` and a listener using
//! the HRTF, three sources with Doppler and cones walking round it, rendered
//! by a capture sink.
//!
//! Ways it could fail: a feature's state is left out of the world (a
//! velocity from outside the snapshot, a direction the mixer keeps alone),
//! so a second run or a restore mid-walk renders other samples or stream.

use crate::support;

use og_audio::schema::panning_model;
use og_audio::{
    AudioLink, AudioListener, AudioSource, AudioStream, HeadlessMode, HeadlessSink, SoundBank,
    Transform,
};
use og_core::glam::Vec3;
use og_core::math::quat_from_axis_angle;
use og_ecs::{Entity, World};
use support::*;

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
                        inner_angle: 45.0f32.to_radians(),
                        outer_angle: 120.0f32.to_radians(),
                        outer_gain: 0.1,
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
