//! Parented audio (row 86): a source, the listener and a reverb zone on a
//! child entity are placed at their `og.Transform` composed down
//! `og.Parent` (`og_core::math::world_pose`, the helper physics places soft
//! bodies with), as the renderer composes `og.MatrixWorld`.
//!
//! Ways it could fail, and the test that catches each:
//! - a child source is heard at its local transform (the world origin plus
//!   its offset), not where its parent carries it, or the composition
//!   differs from an unparented source at the same place →
//!   `a_source_on_a_child_follows_its_moving_parent`
//! - a listener under a camera rig hears from its own local transform, or
//!   ignores the rig's turn → `a_listener_under_a_rig_hears_from_the_rig`
//! - a zone under a parent stays at the origin, or ignores its parent's
//!   scale → `a_zone_under_a_parent_is_where_its_parent_puts_it`
//!
//! Broken once by reading each entity's own `og.Transform` again
//! (`Places::world` returning `t`): all three fail. The first also holds
//! the Doppler shift a carried source gets (its pitch bits, and the shift
//! itself, which the code before Doppler never sent).

mod support;

use og_audio::{
    AudioListener, AudioSource, AudioStream, Op, Parent, ReverbZone, SoundBank, Transform,
};
use og_core::glam::{Quat, Vec3};
use og_core::math::quat_from_axis_angle;
use og_ecs::{Entity, World};
use support::*;

fn parent(e: Entity) -> Parent {
    Parent {
        entity: e.to_bits(),
    }
}

/// The (gain, pan, cutoff, pitch) bits of every spatial play or params
/// command in one tick's packets (the pitch carries the Doppler shift).
fn spatial(link: &og_audio::AudioLink) -> Vec<(u32, u32, u32, u32)> {
    link.drain()
        .iter()
        .flat_map(|p| p.commands.clone())
        .filter(|c| c.voice != 0 && matches!(c.op, Op::Play | Op::Params))
        .map(|c| {
            (
                c.gain.to_bits(),
                c.pan.to_bits(),
                c.lowpass.to_bits(),
                c.pitch.to_bits(),
            )
        })
        .collect()
}

fn hum(bank: &SoundBank) -> AudioSource {
    let sound = insert(bank, "hum", 1, &synth(48_000, 1, sine(220.0, 0.5)));
    AudioSource {
        looping: true,
        spatial: true,
        ref_distance: 1.0,
        distance_cutoff: 800.0,
        ..AudioSource::new(sound)
    }
}

/// A cart moves from x = -10 to 10 carrying a source 2 m ahead of it
/// (local -Z); a second world places the same source there itself. The
/// two send the same commands, bit for bit, every tick: hard left to hard
/// right, nearest in the middle.
#[test]
fn a_source_on_a_child_follows_its_moving_parent() {
    let bank = SoundBank::new();
    let source = hum(&bank);
    let run = |parented: bool| -> (Vec<(u32, u32, u32, u32)>, u64) {
        let (mut world, link) = world_with(&bank, |w| {
            w.register::<AudioListener>().unwrap();
            w.register::<Parent>().unwrap();
        });
        world
            .spawn((Transform::default(), AudioListener::default()))
            .unwrap();
        let cart = world.spawn((Transform::at(Vec3::X * -10.0),)).unwrap();
        let e = if parented {
            world
                .spawn((Transform::at(Vec3::NEG_Z * 2.0), parent(cart), source))
                .unwrap()
        } else {
            world
                .spawn((Transform::at(Vec3::new(-10.0, 0.0, -2.0)), source))
                .unwrap()
        };
        link.attach();
        let mut sent = Vec::new();
        for t in 0..120u64 {
            let x = -10.0 + 20.0 * t as f32 / 119.0;
            if parented {
                world.insert(cart, Transform::at(Vec3::X * x)).unwrap();
            } else {
                world
                    .insert(e, Transform::at(Vec3::new(x, 0.0, -2.0)))
                    .unwrap();
            }
            world.step().unwrap();
            sent.extend(spatial(&link));
        }
        (sent, world.resource::<AudioStream>().unwrap().hash)
    };
    let (carried, hash) = run(true);
    let (placed, placed_hash) = run(false);
    assert_eq!(carried.len(), 120, "a moving source sends every tick");
    assert_eq!(carried, placed, "a carried source is heard where it is");
    assert_eq!(hash, placed_hash, "the same stream");
    assert_eq!(run(true).1, hash, "deterministic");
    let pans: Vec<f32> = carried.iter().map(|c| f32::from_bits(c.1)).collect();
    assert!(pans[0] < -0.95 && pans[119] > 0.95, "{pans:?}");
    let gains: Vec<f32> = carried.iter().map(|c| f32::from_bits(c.0)).collect();
    assert!(
        gains[60] > 3.0 * gains[0],
        "nearest in the middle: {gains:?}"
    );
    // Carried at 10 m/s, its Doppler shift is the placed source's: up
    // while it comes, down once it has passed.
    let pitches: Vec<f32> = carried.iter().map(|c| f32::from_bits(c.3)).collect();
    assert!(
        pitches[30] > 1.01 && pitches[90] < 0.99,
        "Doppler down og.Parent: {pitches:?}"
    );
}

/// A camera rig at (10, 0, 0) turned a quarter left (+90° about Y: the
/// listener under it looks down -X, its right is -Z) hears a source at
/// (10, 0, 3) on its left at 3 m; from the listener's own transform (the
/// origin, looking down -Z) it would be on the right at 10.4 m. Moving
/// the rig moves the ear.
#[test]
fn a_listener_under_a_rig_hears_from_the_rig() {
    let bank = SoundBank::new();
    let (mut world, link) = world_with(&bank, |w| {
        w.register::<AudioListener>().unwrap();
        w.register::<Parent>().unwrap();
    });
    let turn = quat_from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    let rig = world
        .spawn((Transform {
            position: Vec3::X * 10.0,
            quaternion: turn,
            ..Transform::default()
        },))
        .unwrap();
    world
        .spawn((Transform::default(), parent(rig), AudioListener::default()))
        .unwrap();
    world
        .spawn((
            Transform::at(Vec3::new(10.0, 0.0, 3.0)),
            AudioSource {
                distance_cutoff: 0.0,
                ..hum(&bank)
            },
        ))
        .unwrap();
    link.attach();
    world.step().unwrap();
    let first = spatial(&link);
    assert_eq!(first.len(), 1);
    let (gain, pan) = (f32::from_bits(first[0].0), f32::from_bits(first[0].1));
    assert!(pan < -0.99, "on the rig's left: {pan}");
    assert!((gain - 1.0 / 3.0).abs() < 1e-4, "3 m away: {gain}");
    // The rig walks to x = 4 and turns back to face -Z: the source is now
    // 6 m to its right and 3 m behind.
    world
        .insert(rig, Transform::at(Vec3::new(4.0, 0.0, 0.0)))
        .unwrap();
    world.step().unwrap();
    let moved = spatial(&link);
    assert_eq!(moved.len(), 1, "the ear moved, so the source's params did");
    let (gain, pan) = (f32::from_bits(moved[0].0), f32::from_bits(moved[0].1));
    let d = Vec3::new(6.0, 0.0, 3.0).length();
    assert!((pan - 6.0 / d).abs() < 1e-5, "{pan}");
    assert!((gain - 1.0 / d).abs() < 1e-4, "{gain}");
}

/// A crypt's zone, a sphere of radius 1 on a child of a hall at (20, 0, 0)
/// scaled 2: the listener at the origin is outside it (weight 0; at its
/// own transform the zone would hold the origin), 1.5 m from the hall it
/// is inside (the hall's scale makes the radius 2), and when the hall
/// moves to the origin the zone comes with it.
#[test]
fn a_zone_under_a_parent_is_where_its_parent_puts_it() {
    let bank = SoundBank::new();
    let (mut world, _link) = world_with(&bank, |w| {
        w.register::<AudioListener>().unwrap();
        w.register::<Parent>().unwrap();
    });
    let ear = world
        .spawn((Transform::default(), AudioListener::default()))
        .unwrap();
    let hall = world
        .spawn((Transform {
            position: Vec3::X * 20.0,
            quaternion: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        },))
        .unwrap();
    let zone = world
        .spawn((
            Transform::default(),
            parent(hall),
            ReverbZone {
                radius: 1.0,
                fade: 0.0,
                ..ReverbZone::default()
            },
        ))
        .unwrap();
    let weight = |world: &mut World| {
        world.step().unwrap();
        world.get::<ReverbZone>(zone).unwrap().weight
    };
    assert_eq!(
        weight(&mut world),
        0.0,
        "the zone is at the hall, not the origin"
    );
    world
        .insert(ear, Transform::at(Vec3::new(21.5, 0.0, 0.0)))
        .unwrap();
    assert_eq!(weight(&mut world), 1.0, "inside the scaled zone");
    world.insert(hall, Transform::default()).unwrap();
    assert_eq!(weight(&mut world), 0.0, "the hall left with its zone");
    world.insert(ear, Transform::at(Vec3::Y * 0.5)).unwrap();
    assert_eq!(weight(&mut world), 1.0, "and the zone is where the hall is");
}
