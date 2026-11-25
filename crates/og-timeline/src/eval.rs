//! Evaluation at a time: every value a timeline sets is a function of the
//! time alone (keys eased between, held outside), never a sum over ticks,
//! so a seek to `t` and playing to `t` give the same values.

use crate::model::*;
use og_core::ease::Ease;
use og_core::glam::{Quat, Vec3};
use og_core::math;

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

/// og-core's one deterministic slerp ([`math::quat_slerp`]), kept unit.
fn slerp(a: Quat, b: Quat, u: f32) -> Quat {
    math::quat_slerp(a, b, u).normalize()
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

/// How much of a camera track's own pose shows at `t` of a timeline of
/// `duration`, from 0 (the camera it took, or hands back) to 1: its
/// `blendIn` from the start and its `blendOut` before the end, cubic in and
/// out (a Sequencer camera cut's blend).
pub fn camera_weight(track: &CameraTrack, t: f32, duration: f32) -> f32 {
    let part = |span: f32, into: f32| {
        if span > 0.0 {
            Ease::CubicInOut.apply((into / span).clamp(0.0, 1.0))
        } else {
            1.0
        }
    };
    part(track.blend_in, t) * part(track.blend_out, duration - t)
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

/// The video key playing at `t` and the video's time then: `from + (t −
/// key) · speed` (`None` before the first key).
pub fn video_at(track: &VideoTrack, t: f32) -> Option<(usize, f32)> {
    video_at_f64(track, f64::from(t)).map(|(i, v)| (i, v as f32))
}

/// Exact `f64` form of [`video_at`], avoiding `f32` precision drift in long
/// (>27 min) video timelines.
pub fn video_at_f64(track: &VideoTrack, t: f64) -> Option<(usize, f64)> {
    let i = track.keys.iter().rposition(|k| f64::from(k.time) <= t)?;
    let k = &track.keys[i];
    Some((
        i,
        f64::from(k.from) + (t - f64::from(k.time)) * f64::from(k.speed),
    ))
}

/// The key of a `scene` track whose scene is in the world at `t`: the last
/// at or before it, unless that one names none (`None` before the first
/// key too).
pub fn scene_at(keys: &[SceneKey], t: f32) -> Option<usize> {
    let i = keys.iter().rposition(|k| k.time <= t)?;
    (!keys[i].scene.is_empty()).then_some(i)
}

/// A hotspot at a time: its id, bounding box (x, y, width, height in
/// fractions of the video frame), shape, kind, and label.
#[derive(Clone, Debug, PartialEq)]
pub struct HotspotAt<'a> {
    pub id: &'a str,
    pub rect: [f32; 4],
    pub shape: HotspotShape,
    pub kind: HotspotKind,
    pub label: &'a str,
}

/// The hotspots of a track at `t`, in the order their ids first appear:
/// each from its key at or before `t` (none before its first or from a
/// key `off`), moving with the next key's ease to its next key and held
/// after its last.
pub fn hotspots_at(keys: &[HotspotKey], t: f32) -> Vec<HotspotAt<'_>> {
    let mut ids: Vec<&str> = Vec::new();
    for k in keys {
        if !ids.contains(&k.id.as_str()) {
            ids.push(&k.id);
        }
    }
    let mut out = Vec::new();
    for id in ids {
        let mine = || keys.iter().filter(move |k| k.id == id);
        let Some(a) = mine().rfind(|k| k.time <= t) else {
            continue;
        };
        if a.off {
            continue;
        }
        // Toward its next key, unless that one ends it.
        let rect = match mine().find(|k| k.time > t) {
            Some(b) if !b.off => {
                let u = b.ease.apply((t - a.time) / (b.time - a.time));
                std::array::from_fn(|i| a.rect[i] + (b.rect[i] - a.rect[i]) * u)
            }
            _ => a.rect,
        };
        out.push(HotspotAt {
            id,
            rect,
            shape: a.shape,
            kind: a.kind,
            label: &a.label,
        });
    }
    out
}

/// The caption showing at `t` in `locale` ([`CaptionTrack::keys_in`]): the
/// latest started that has not ended.
pub fn caption_at<'a>(
    track: &'a CaptionTrack,
    locale: &str,
    t: f32,
) -> Option<(usize, &'a CaptionKey)> {
    let showing = |(_, k): &(usize, &CaptionKey)| k.time <= t && t < k.end;
    track.keys_in(locale).iter().enumerate().rev().find(showing)
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
