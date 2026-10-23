//! Evaluation at a time: every value a timeline sets is a function of the
//! time alone (keys eased between, held outside), never a sum over ticks,
//! so a seek to `t` and playing to `t` give the same values.

use crate::model::*;
use og_core::glam::{Quat, Vec3};
use og_core::math;
use og_tween::Ease;

/// A channel's value at `t`: held at the first key that has it before it,
/// at the last after it; between two such keys, mixed by the later key's
/// ease. `None` when no key has the channel.
pub fn channel<K, T: Copy>(
    keys: &[K],
    t: f32,
    time: impl Fn(&K) -> f32,
    ease: impl Fn(&K) -> Ease,
    get: impl Fn(&K) -> Option<T>,
    mix: impl Fn(T, T, f32) -> T,
) -> Option<T> {
    let mut before: Option<(f32, T)> = None;
    for k in keys {
        let Some(v) = get(k) else { continue };
        let kt = time(k);
        if kt <= t {
            before = Some((kt, v));
            continue;
        }
        return Some(match before {
            None => v,
            Some((at, a)) => {
                let span = kt - at;
                let u = if span > 0.0 { (t - at) / span } else { 1.0 };
                mix(a, v, ease(k).apply(u))
            }
        });
    }
    before.map(|(_, v)| v)
}

pub fn lerp3(a: Vec3, b: Vec3, u: f32) -> Vec3 {
    a + (b - a) * u
}

pub fn lerp(a: f32, b: f32, u: f32) -> f32 {
    a + (b - a) * u
}

/// Spherical interpolation on the shorter arc, with og-core's deterministic
/// trigonometry (a normalised lerp when the two are nearly equal).
pub fn slerp(a: Quat, b: Quat, u: f32) -> Quat {
    let mut b = b;
    let mut d = a.dot(b);
    if d < 0.0 {
        b = -b;
        d = -d;
    }
    if d > 0.9995 {
        return (a + (b - a) * u).normalize();
    }
    let theta = math::atan2((1.0 - d * d).max(0.0).sqrt(), d);
    let s = math::sin(theta);
    let (s0, s1) = (math::sin((1.0 - u) * theta) / s, math::sin(u * theta) / s);
    (a * s0 + b * s1).normalize()
}

/// What a camera track asks for at a time (in its mode's terms: world
/// positions, a rail parameter, or offsets from a followed entity).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraAt {
    pub position: Option<Vec3>,
    pub target: Option<Vec3>,
    pub at: Option<f32>,
    pub fov: Option<f32>,
}

pub fn camera_at(track: &CameraTrack, t: f32) -> CameraAt {
    let k = &track.keys;
    let (time, ease) = (|k: &CameraKey| k.time, |k: &CameraKey| k.ease);
    CameraAt {
        position: channel(k, t, time, ease, |k| k.position, lerp3),
        target: channel(k, t, time, ease, |k| k.target, lerp3),
        at: channel(k, t, time, ease, |k| k.at, lerp),
        fov: channel(k, t, time, ease, |k| k.fov, lerp),
    }
}

/// The parts of a transform a track sets at a time.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransformAt {
    pub position: Option<Vec3>,
    pub quaternion: Option<Quat>,
    pub scale: Option<Vec3>,
}

pub fn transform_at(track: &TransformTrack, t: f32) -> TransformAt {
    let k = &track.keys;
    let (time, ease) = (|k: &TransformKey| k.time, |k: &TransformKey| k.ease);
    TransformAt {
        position: channel(k, t, time, ease, |k| k.position, lerp3),
        quaternion: channel(k, t, time, ease, |k| k.quaternion, slerp),
        scale: channel(k, t, time, ease, |k| k.scale, lerp3),
    }
}

/// The animation key in effect at `t` (the last at or before it) and the
/// clip's own time: `(t - key.time) * speed`, unwrapped.
pub fn clip_at(track: &AnimationTrack, t: f32) -> Option<(usize, f32)> {
    let i = track.keys.iter().rposition(|k| k.time <= t)?;
    let k = &track.keys[i];
    Some((i, (t - k.time) * k.speed))
}

/// The caption showing at `t`: the latest started that has not ended.
pub fn caption_at(track: &CaptionTrack, t: f32) -> Option<&CaptionKey> {
    track.keys.iter().rev().find(|k| k.time <= t && t < k.end)
}

/// How a transition track covers the screen at a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoverAt {
    pub kind: TransitionKind,
    pub from: u32,
    pub color: Vec3,
    /// 0 clear to 1 covered.
    pub amount: f32,
    /// The cover is opening (it covered before).
    pub revealing: bool,
}

/// The cover at `t`, from the last transition at or before it; `None` when
/// nothing covers the screen.
pub fn cover_at(keys: &[TransitionKey], t: f32) -> Option<CoverAt> {
    let k = keys.iter().rev().find(|k| k.time <= t)?;
    let p = if k.duration > 0.0 {
        ((t - k.time) / k.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let held = k.duration > 0.0 && p < 1.0;
    let (amount, revealing) = match (k.kind, k.mode) {
        (TransitionKind::Cut, TransitionMode::Out) => (1.0, false),
        (TransitionKind::Cut, _) => (if held { 1.0 } else { 0.0 }, false),
        (_, TransitionMode::Out) => (p, false),
        (_, TransitionMode::In) => (1.0 - p, true),
        (_, TransitionMode::Through) if p < 0.5 => (p * 2.0, false),
        (_, TransitionMode::Through) => (2.0 - p * 2.0, true),
    };
    (amount > 0.0).then_some(CoverAt {
        kind: k.kind,
        from: k.from,
        color: k.color,
        amount: amount.min(1.0),
        revealing,
    })
}

/// A music key's gain at `t` (its volume through its fades), or `None`
/// outside `time..end` (`end` defaults to the timeline's duration).
pub fn music_gain(k: &MusicKey, t: f32, duration: f32) -> Option<f32> {
    let end = k.end.unwrap_or(duration);
    if t < k.time || t >= end {
        return None;
    }
    let fin = if k.fade_in > 0.0 {
        ((t - k.time) / k.fade_in).min(1.0)
    } else {
        1.0
    };
    let fout = if k.fade_out > 0.0 {
        ((end - t) / k.fade_out).min(1.0)
    } else {
        1.0
    };
    Some(k.volume * fin * fout)
}

/// Whether a key at `time` is passed going from `from` (exclusive) to `to`
/// (inclusive).
pub fn crossed(time: f32, from: f64, to: f64) -> bool {
    let t = f64::from(time);
    t > from && t <= to
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(time: f32, x: Option<f32>, ease: Ease) -> TransformKey {
        TransformKey {
            time,
            ease,
            position: x.map(|x| Vec3::new(x, 0.0, 0.0)),
            ..TransformKey::default()
        }
    }

    #[test]
    fn channels_hold_outside_and_ease_by_the_later_key() {
        let tr = TransformTrack {
            entity: "e".into(),
            keys: vec![
                key(1.0, Some(0.0), Ease::Linear),
                key(2.0, None, Ease::Linear),
                key(3.0, Some(10.0), Ease::QuadIn),
            ],
        };
        let x = |t| transform_at(&tr, t).position.unwrap().x;
        assert_eq!(x(0.0), 0.0);
        // Half way from 1 to 3 (the key at 2 has no position), quadIn: 0.25.
        assert!((x(2.0) - 2.5).abs() < 1e-6);
        assert_eq!(x(9.0), 10.0);
        assert!(transform_at(&tr, 2.0).scale.is_none());
    }

    #[test]
    fn slerp_takes_the_short_way_and_lands_exactly() {
        let a = Quat::IDENTITY;
        let b = crate::model::euler_degrees(Vec3::new(0.0, 90.0, 0.0));
        let m = slerp(a, b, 0.5);
        let want = crate::model::euler_degrees(Vec3::new(0.0, 45.0, 0.0));
        assert!(m.dot(want).abs() > 1.0 - 1e-6, "{m:?}");
        assert!(slerp(a, -b, 0.5).dot(want).abs() > 1.0 - 1e-6);
        assert!(slerp(a, b, 1.0).dot(b).abs() > 1.0 - 1e-6);
    }

    #[test]
    fn covers_through_in_out_and_cut() {
        let k = |time, kind, mode, duration| TransitionKey {
            time,
            kind,
            mode,
            duration,
            color: Vec3::ZERO,
            from: 0,
        };
        let keys = vec![
            k(0.0, TransitionKind::Fade, TransitionMode::In, 1.0),
            k(2.0, TransitionKind::Wipe, TransitionMode::Through, 1.0),
            k(4.0, TransitionKind::Cut, TransitionMode::Through, 0.5),
            k(6.0, TransitionKind::Fade, TransitionMode::Out, 2.0),
        ];
        let a = |t| cover_at(&keys, t).map_or(0.0, |c| c.amount);
        assert_eq!(a(0.0), 1.0);
        assert!((a(0.25) - 0.75).abs() < 1e-6 && cover_at(&keys, 0.25).unwrap().revealing);
        assert_eq!(a(1.5), 0.0);
        assert!((a(2.25) - 0.5).abs() < 1e-6 && !cover_at(&keys, 2.25).unwrap().revealing);
        assert!((a(2.75) - 0.5).abs() < 1e-6 && cover_at(&keys, 2.75).unwrap().revealing);
        assert_eq!(a(4.2), 1.0);
        assert_eq!(a(4.5), 0.0);
        assert!((a(7.0) - 0.5).abs() < 1e-6);
        assert_eq!(a(9.0), 1.0, "an `out` holds the screen covered");
    }

    #[test]
    fn music_fades_in_and_out_by_the_time_alone() {
        let k = MusicKey {
            time: 1.0,
            sound: "m".into(),
            volume: 0.5,
            fade_in: 2.0,
            fade_out: 1.0,
            end: Some(6.0),
            looping: true,
            bus: 1,
        };
        assert_eq!(music_gain(&k, 0.5, 8.0), None);
        assert_eq!(music_gain(&k, 2.0, 8.0), Some(0.25));
        assert_eq!(music_gain(&k, 4.0, 8.0), Some(0.5));
        assert_eq!(music_gain(&k, 5.5, 8.0), Some(0.25));
        assert_eq!(music_gain(&k, 6.0, 8.0), None);
        assert!(crossed(2.0, 1.9, 2.0) && !crossed(2.0, 2.0, 2.1));
    }
}
