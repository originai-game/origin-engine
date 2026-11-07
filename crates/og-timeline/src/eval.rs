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
    pub roll: Option<f32>,
    pub focus: Option<f32>,
    pub focus_range: Option<f32>,
}

pub fn camera_at(track: &CameraTrack, t: f32) -> CameraAt {
    let k = &track.keys;
    let (time, ease) = (|k: &CameraKey| k.time, |k: &CameraKey| k.ease);
    CameraAt {
        position: channel(k, t, time, ease, |k| k.position, lerp3),
        target: channel(k, t, time, ease, |k| k.target, lerp3),
        at: channel(k, t, time, ease, |k| k.at, lerp),
        fov: channel(k, t, time, ease, |k| k.fov, lerp),
        roll: channel(k, t, time, ease, |k| k.roll, lerp),
        focus: channel(k, t, time, ease, |k| k.focus, lerp),
        focus_range: channel(k, t, time, ease, |k| k.focus_range, lerp),
    }
}

/// The entity a camera keeps in focus at `t`: the last key at or before it
/// that names one, unless a later such key sets a `focus` distance instead.
pub fn focus_on_at(track: &CameraTrack, t: f32) -> Option<&str> {
    let mut on = None;
    for k in track.keys.iter().take_while(|k| k.time <= t) {
        if let Some(e) = &k.focus_on {
            on = Some(e.as_str());
        } else if k.focus.is_some() {
            on = None;
        }
    }
    on
}

/// How a field moves between two keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mix {
    /// Each number eased (scalars, vectors, colours).
    Lerp,
    /// A quaternion: spherical, on the shorter arc.
    Slerp,
    /// Held until the next key (bools, enums, integers, visibility).
    Step,
}

/// A field track's value of `field` at `t`: eased between the keys that
/// set it by the later one's ease (stepped for [`Mix::Step`]), held outside
/// them. Numbers of different lengths, or a word against numbers, step.
pub fn field_at(track: &FieldsTrack, field: &str, t: f32, mix: Mix) -> Option<FieldValue> {
    let get = |k: &FieldKey| {
        k.values
            .iter()
            .find(|(n, _)| n == field)
            .map(|(_, v)| v.clone())
    };
    let mut before: Option<(f32, FieldValue)> = None;
    for k in &track.keys {
        let Some(v) = get(k) else { continue };
        if k.time <= t {
            before = Some((k.time, v));
            continue;
        }
        let Some((at, a)) = before else {
            return Some(v);
        };
        let span = k.time - at;
        let u = if span > 0.0 { (t - at) / span } else { 1.0 };
        let u = k.ease.apply(u);
        return Some(match (&a, &v, mix) {
            (FieldValue::Numbers(x), FieldValue::Numbers(y), Mix::Slerp)
                if x.len() == 4 && y.len() == 4 =>
            {
                let q = slerp(
                    Quat::from_xyzw(x[0], x[1], x[2], x[3]).normalize(),
                    Quat::from_xyzw(y[0], y[1], y[2], y[3]).normalize(),
                    u,
                );
                FieldValue::Numbers(vec![q.x, q.y, q.z, q.w])
            }
            (FieldValue::Numbers(x), FieldValue::Numbers(y), Mix::Lerp | Mix::Slerp)
                if x.len() == y.len() =>
            {
                FieldValue::Numbers(x.iter().zip(y).map(|(a, b)| lerp(*a, *b, u)).collect())
            }
            _ => a,
        });
    }
    before.map(|(_, v)| v)
}

/// The nested timeline playing at `t` of a `timeline` track: its key, when
/// that key's span ends (the next key, else `end`), and the child's own
/// time: `(t - key) * speed`, wrapped by `duration` when it loops, else
/// held at it.
pub fn nested_at(
    keys: &[NestedKey],
    t: f32,
    end: f32,
    duration: impl Fn(&NestedKey) -> f32,
) -> Option<(usize, f32, f32)> {
    let i = keys.iter().rposition(|k| k.time <= t)?;
    let k = &keys[i];
    let until = keys.get(i + 1).map_or(end, |n| n.time);
    Some((i, until, child_time(k, t, duration(k))))
}

/// A nested timeline's own time at the parent's `t`.
pub fn child_time(k: &NestedKey, t: f32, duration: f32) -> f32 {
    let ct = ((t - k.time) * k.speed).max(0.0);
    if k.looping && duration > 0.0 {
        ct.rem_euclid(duration)
    } else {
        ct.min(duration)
    }
}

/// The parent's playback span `(a, b]` as the child's spans: shifted to the
/// key, scaled by its speed, cut to `[key, until)`, and split at each wrap
/// of a looping child (each a span of its own, the next from just before
/// 0 so a key at 0 fires again).
pub fn child_spans(k: &NestedKey, until: f32, duration: f32, a: f64, b: f64) -> Vec<(f64, f64)> {
    let (start, until) = (f64::from(k.time), f64::from(until));
    let lo = a.max(start - 1e-9);
    let hi = b.min(until);
    if hi <= lo {
        return Vec::new();
    }
    let speed = f64::from(k.speed);
    let map = |x: f64| {
        if x < start {
            -1e-9
        } else {
            (x - start) * speed
        }
    };
    let (ca, cb) = (map(lo), map(hi));
    let d = f64::from(duration);
    if !k.looping || d <= 0.0 {
        let cb = cb.min(d);
        return if cb > ca { vec![(ca, cb)] } else { Vec::new() };
    }
    let mut out = Vec::new();
    let mut lap = (ca.max(0.0) / d).floor();
    let mut span_from = ca - lap * d;
    loop {
        let off = lap * d;
        let to = cb.min(off + d) - off;
        if to > span_from {
            out.push((span_from, to));
        }
        if cb <= off + d || out.len() > 64 {
            break;
        }
        lap += 1.0;
        span_from = -1e-9;
    }
    out
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
        // Half way from 1 to 3 (the key at 2 has no position), power1.in: 0.25.
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

    /// A field eases by the later key's ease between the keys that set it
    /// (another field's keys in between do not split it), steps when asked,
    /// slerps a quaternion, and a word steps.
    #[test]
    fn fields_ease_between_their_own_keys_and_step_or_slerp_by_kind() {
        let n = |v: &[f32]| FieldValue::Numbers(v.to_vec());
        let key = |time: f32, ease: Ease, values: Vec<(&str, FieldValue)>| FieldKey {
            time,
            ease,
            values: values
                .into_iter()
                .map(|(a, b)| (a.to_string(), b))
                .collect(),
        };
        let t = FieldsTrack {
            kind: FieldsKind::Light,
            entity: "lamp".into(),
            index: 0,
            keys: vec![
                key(
                    0.0,
                    Ease::Linear,
                    vec![
                        ("intensity", n(&[0.0])),
                        ("castShadow", FieldValue::Word("false".into())),
                    ],
                ),
                key(1.0, Ease::Linear, vec![("color", n(&[1.0, 0.0, 0.0]))]),
                key(
                    4.0,
                    Ease::QuadIn,
                    vec![
                        ("intensity", n(&[8.0])),
                        ("castShadow", FieldValue::Word("true".into())),
                    ],
                ),
            ],
        };
        // Halfway from 0 to 4 s, power1.in: 8 × 0.25.
        assert_eq!(field_at(&t, "intensity", 2.0, Mix::Lerp), Some(n(&[2.0])));
        assert_eq!(field_at(&t, "intensity", 2.0, Mix::Step), Some(n(&[0.0])));
        assert_eq!(field_at(&t, "intensity", 9.0, Mix::Lerp), Some(n(&[8.0])));
        assert_eq!(
            field_at(&t, "color", 0.0, Mix::Lerp),
            Some(n(&[1.0, 0.0, 0.0])),
            "held before its first key"
        );
        assert_eq!(
            field_at(&t, "castShadow", 3.99, Mix::Lerp),
            Some(FieldValue::Word("false".into()))
        );
        assert_eq!(
            field_at(&t, "castShadow", 4.0, Mix::Lerp),
            Some(FieldValue::Word("true".into()))
        );
        assert_eq!(field_at(&t, "range", 2.0, Mix::Lerp), None);
        let q = FieldsTrack {
            keys: vec![
                key(0.0, Ease::Linear, vec![("q", n(&[0.0, 0.0, 0.0, 1.0]))]),
                key(1.0, Ease::Linear, vec![("q", n(&[0.0, 1.0, 0.0, 0.0]))]),
            ],
            ..t
        };
        // Halfway through a 180° turn about y: 90°, unit length (a lerp is 0.707 long).
        let Some(FieldValue::Numbers(h)) = field_at(&q, "q", 0.5, Mix::Slerp) else {
            panic!()
        };
        assert!(
            (h[1] - 0.70710677).abs() < 1e-5 && (h[3] - 0.70710677).abs() < 1e-5,
            "{h:?}"
        );
    }

    /// A nested timeline's time is the parent's shifted to its key and
    /// scaled by its speed, held at its end or wrapped when it loops; the
    /// parent's playback spans map to the child's, a key at the child's 0
    /// firing on its first tick and again on each lap.
    #[test]
    fn nested_time_and_spans_follow_the_key() {
        let k = NestedKey {
            time: 2.0,
            play: "c".into(),
            speed: 2.0,
            looping: false,
        };
        assert_eq!(child_time(&k, 1.0, 3.0), 0.0);
        assert_eq!(child_time(&k, 2.5, 3.0), 1.0);
        assert_eq!(child_time(&k, 9.0, 3.0), 3.0, "held at its end");
        // (1.9, 2.1] crosses the key: the child's first span starts just before 0.
        let s = child_spans(&k, 10.0, 3.0, 1.9, 2.1);
        assert_eq!(s.len(), 1);
        assert!(s[0].0 < 0.0 && (s[0].1 - 0.2).abs() < 1e-9, "{s:?}");
        assert!(crossed(0.0, s[0].0, s[0].1));
        assert!(
            child_spans(&k, 10.0, 3.0, 1.0, 1.5).is_empty(),
            "before its key"
        );
        assert!(
            child_spans(&k, 3.0, 3.0, 3.0, 3.5).is_empty(),
            "after the next key"
        );
        let l = NestedKey { looping: true, ..k };
        assert_eq!(
            child_time(&l, 4.0, 3.0),
            1.0,
            "4 s of child time wraps to 1"
        );
        // (3.4, 3.6]: child 2.8 to 3.2 wraps: (2.8, 3] then (-0, 0.2].
        let s = child_spans(&l, 10.0, 3.0, 3.4, 3.6);
        assert_eq!(s.len(), 2, "{s:?}");
        assert!((s[0].0 - 2.8).abs() < 1e-9 && (s[0].1 - 3.0).abs() < 1e-9);
        assert!(s[1].0 < 0.0 && (s[1].1 - 0.2).abs() < 1e-9 && crossed(0.0, s[1].0, s[1].1));
    }
}
