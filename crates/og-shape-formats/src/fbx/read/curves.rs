//! Animation curves and layers: keys with their interpolation, blended across layers, and blend-shape weights from channels.

use super::*;

/// A model's translation, rotation and scaling as an animation's layers
/// leave them (`docs/research.md` §45).
pub(crate) struct Layered {
    pub(crate) t: DVec3,
    /// Euler degrees, until a layer composes a turn onto them.
    pub(crate) r: DVec3,
    /// The rotation once layers were composed as turns.
    pub(crate) turned: Option<Affine>,
    pub(crate) s: DVec3,
}

/// One layer of an animation stack: how it lies on those under it.
pub(crate) struct Layer {
    /// 0 adds, 2 mixes towards its value by its weight, others replace.
    pub(crate) mode: i64,
    /// 0..=1.
    pub(crate) weight: f64,
    /// The weight over time, in percent, where the file animates it.
    pub(crate) weight_curve: Option<Curve>,
    /// Rotations add as turns (not angle by angle); scalings multiply.
    pub(crate) turns: bool,
    pub(crate) multiplies: bool,
    /// Per model: its translation, rotation and scaling curves, an axis
    /// each, and the value an axis without a curve has in this layer.
    pub(crate) driven: BTreeMap<i64, [Channel; 3]>,
    /// Per blend-shape channel: its `DeformPercent` curve, and its value
    /// in this layer without one.
    pub(crate) shapes: BTreeMap<i64, (Option<Curve>, f64)>,
}

/// A property's three curves in a layer, and its values without them.
#[derive(Default)]
pub(crate) struct Channel {
    pub(crate) on: bool,
    pub(crate) curves: [Option<Curve>; 3],
    pub(crate) rest: [f64; 3],
}

impl Layered {
    /// Lays `value` of property `slot` (0 translation, 1 rotation, 2
    /// scaling) on, as `layer` does; the stack's first layer replaces.
    pub(crate) fn lay(
        &mut self,
        first: bool,
        layer: &Layer,
        time: f64,
        slot: usize,
        value: DVec3,
        order: i64,
    ) {
        let w = layer
            .weight_curve
            .as_ref()
            .map_or(layer.weight, |c| (c.at(time) / 100.0).clamp(0.0, 1.0));
        let mode = if first { 1 } else { layer.mode };
        let power = |v: f64, e: f64| {
            if e <= 0.0 {
                1.0
            } else if e >= 1.0 {
                v
            } else {
                double::pow(v.abs(), e) * if v < 0.0 { -1.0 } else { 1.0 }
            }
        };
        match (slot, mode) {
            (0, 0) => self.t += value * w,
            (0, 2) => self.t = self.t * (1.0 - w) + value * w,
            (0, _) => self.t = value,
            (2, 0) if layer.multiplies => {
                self.s *= DVec3::new(power(value.x, w), power(value.y, w), power(value.z, w));
            }
            (2, 0) => self.s += value * w,
            (2, 2) if layer.multiplies => {
                let mix = |a: f64, b: f64| power(a, 1.0 - w) * power(b, w);
                self.s = DVec3::new(
                    mix(self.s.x, value.x),
                    mix(self.s.y, value.y),
                    mix(self.s.z, value.z),
                );
            }
            (2, 2) => self.s = self.s * (1.0 - w) + value * w,
            (2, _) => self.s = value,
            (_, 0 | 2) if layer.turns => {
                let under = self.turned.unwrap_or_else(|| euler(self.r, order));
                let over = euler(value, order);
                // Adding: the layer's turn, part of it, after those
                // under. Mixing: part of the way from those to it.
                let step = if mode == 0 {
                    part_turn(&over, w)
                } else {
                    part_turn(&transpose(&under).then(&over), w)
                };
                self.turned = Some(under.then(&step));
            }
            (_, 0) => self.r += value * w,
            (_, 2) => self.r = self.r * (1.0 - w) + value * w,
            _ => {
                self.r = value;
                self.turned = None;
            }
        }
    }
}

/// How a curve runs from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Ease {
    /// The key's value, until the next.
    Hold,
    /// The next key's value, from this one on.
    HoldNext,
    /// Straight.
    Line,
    /// A cubic Bézier in time and value, by the handles either side.
    Cubic,
}

/// One number of a model's property over time (an `AnimationCurve`): its
/// keys, and for each how it runs to the next (`docs/research.md` §45).
#[derive(Clone, Debug)]
pub(crate) struct Curve {
    /// Key times, seconds, rising.
    pub(crate) times: Vec<f64>,
    pub(crate) values: Vec<f64>,
    pub(crate) ease: Vec<Ease>,
    /// Per key, the handle leaving it: seconds and value from the key.
    pub(crate) right: Vec<(f64, f64)>,
    /// Per key, the handle arriving at it: seconds and value before it.
    pub(crate) left: Vec<(f64, f64)>,
}

pub(crate) const KEY_CONSTANT: i64 = 0x2;

pub(crate) const KEY_CUBIC: i64 = 0x8;

pub(crate) const KEY_AUTO_OR_NEXT: i64 = 0x100;

pub(crate) const KEY_TCB: i64 = 0x200;

pub(crate) const KEY_USER: i64 = 0x400;

pub(crate) const KEY_CLAMP: i64 = 0x1000;

pub(crate) const KEY_TIME_INDEPENDENT: i64 = 0x2000;

pub(crate) const KEY_CLAMP_PROGRESSIVE: i64 = 0x4000;

pub(crate) const KEY_WEIGHTED_RIGHT: i64 = 0x100_0000;

pub(crate) const KEY_WEIGHTED_NEXT_LEFT: i64 = 0x200_0000;

impl Curve {
    /// The curve of `times` and `values` with the file's key attributes:
    /// `flags` and four numbers of `data` for each run of keys that
    /// `refs` counts. Without them every key runs straight to the next.
    pub(crate) fn new(
        times: Vec<f64>,
        values: Vec<f64>,
        flags: &[f64],
        data: &[f64],
        refs: &[f64],
    ) -> Curve {
        let n = times.len();
        let third = 1.0 / 3.0;
        let mut c = Curve {
            ease: vec![Ease::Line; n],
            right: vec![(0.0, 0.0); n],
            left: vec![(0.0, 0.0); n],
            times,
            values,
        };
        // Each key's run of attributes.
        let mut run_of = Vec::with_capacity(n);
        for (run, count) in refs.iter().enumerate() {
            for _ in 0..(*count).max(0.0) as usize {
                if run_of.len() < n {
                    run_of.push(run);
                }
            }
        }
        if run_of.len() < n || flags.len() < refs.len() || data.len() < refs.len() * 4 {
            run_of.clear();
        }
        // What the key before left for this key's arriving handle.
        let (mut slope_left, mut weight_left) = (0.0f64, third);
        for i in 0..n {
            let (t, v) = (c.times[i], c.values[i]);
            let before = (i > 0 && t > c.times[i - 1]).then(|| (c.times[i - 1], c.values[i - 1]));
            let after =
                (i + 1 < n && c.times[i + 1] > t).then(|| (c.times[i + 1], c.values[i + 1]));
            let chord = after.map_or(0.0, |(nt, nv)| (nv - v) / (nt - t));
            let (flag, attr) = match run_of.get(i) {
                Some(&run) => (flags[run] as i64, &data[run * 4..run * 4 + 4]),
                None => (0, &[0.0; 4][..]),
            };
            let (mut slope_right, mut next_slope_left) = (attr[0], attr[1]);
            let (mut weight_right, mut next_weight_left) = (third, third);
            // Weights: two decimal fixed-point numbers in the third
            // number's bits.
            let packed = (attr[2] as f32).to_bits();
            if flag & KEY_WEIGHTED_RIGHT != 0 {
                weight_right = f64::from(packed & 0xffff) * 0.0001;
            }
            if flag & KEY_WEIGHTED_NEXT_LEFT != 0 {
                next_weight_left = f64::from(packed >> 16) * 0.0001;
            }
            if flag & KEY_CONSTANT != 0 {
                c.ease[i] = if flag & KEY_AUTO_OR_NEXT != 0 {
                    Ease::HoldNext
                } else {
                    Ease::Hold
                };
                (weight_right, next_weight_left) = (third, third);
                (slope_right, next_slope_left) = (0.0, 0.0);
            } else if flag & KEY_CUBIC != 0 {
                c.ease[i] = Ease::Cubic;
                if flag & KEY_TCB != 0 {
                    // Tension, continuity and bias mix the chords either side.
                    let (tension, continuity, bias) = (attr[0], attr[1], attr[2]);
                    let chord_left = before.map_or(0.0, |(pt, pv)| (v - pv) / (t - pt));
                    let factor = if before.is_some() && after.is_some() {
                        0.5
                    } else {
                        1.0
                    };
                    let k = factor * (1.0 - tension);
                    slope_left = k
                        * ((1.0 + bias) * (1.0 - continuity) * chord_left
                            + (1.0 - bias) * (1.0 + continuity) * chord);
                    slope_right = k
                        * ((1.0 + bias) * (1.0 + continuity) * chord_left
                            + (1.0 - bias) * (1.0 - continuity) * chord);
                    next_slope_left = 0.0;
                    next_weight_left = third;
                } else if flag & KEY_USER == 0 {
                    // Automatic: the slope through the keys either side;
                    // what the file stores is a bias.
                    let auto = |bias: f64| -> f64 {
                        match (before, after) {
                            (Some((pt, pv)), Some((nt, nv))) => {
                                if flag & KEY_CLAMP != 0 && (pv == v || nv == v) {
                                    return 0.0;
                                }
                                let mut slope = (nv - pv) / (nt - pt);
                                let (sl, sr) = ((v - pv) / (t - pt), (nv - v) / (nt - t));
                                if flag & KEY_TIME_INDEPENDENT == 0 {
                                    let delta = (t - pt) / (nt - pt);
                                    slope = slope * 0.5 + (sl * (1.0 - delta) + sr * delta) * 0.5;
                                    let w = bias.abs() / 100.0;
                                    if w > 0.0001 {
                                        let target = if bias > 0.0 { sr } else { sl };
                                        let toward = target - slope;
                                        slope = slope * (1.0 - w) + target * w;
                                        if w > 5.0 {
                                            let sign =
                                                if toward.abs() > 0.00001 { toward } else { bias };
                                            let sign = if sign > 0.0 { 1.0 } else { -1.0 };
                                            slope += (w - 5.0) * (w - 5.0) * sign * 40.0;
                                        }
                                    }
                                }
                                if flag & KEY_CLAMP_PROGRESSIVE != 0 {
                                    // Neither handle passes its neighbour's value.
                                    let sign = if slope >= 0.0 { 1.0 } else { -1.0 };
                                    let reach_left = weight_left * (t - pt);
                                    let reach_right = weight_right * (nt - t);
                                    let mut most = slope * sign;
                                    for limit in [
                                        sign * (v - pv) / reach_left,
                                        sign * (nv - v) / reach_right,
                                    ] {
                                        most = most.min(if limit > 0.0 { limit } else { 0.0 });
                                    }
                                    slope = sign * most;
                                }
                                slope
                            }
                            (side_before, side_after) => {
                                let Some((ot, ov)) = side_before.or(side_after) else {
                                    return 0.0;
                                };
                                if flag & KEY_CLAMP_PROGRESSIVE != 0
                                    || (flag & KEY_CLAMP != 0 && ov == v)
                                {
                                    return 0.0;
                                }
                                let mut slope = (ov - v) / (ot - t);
                                let w = bias.abs() / 100.0 - 5.0;
                                if flag & KEY_TIME_INDEPENDENT == 0 && w > 0.0 {
                                    slope += w * w * if bias > 0.0 { 40.0 } else { -40.0 };
                                }
                                slope
                            }
                        }
                    };
                    if before.is_some()
                        && after.is_some()
                        && (slope_left + slope_right).abs() > 0.0001
                    {
                        (slope_left, slope_right) = (auto(-slope_left), auto(slope_right));
                    } else if before.is_some() && after.is_none() {
                        slope_left = auto(-slope_left);
                        slope_right = slope_left;
                    } else {
                        slope_right = auto(slope_right);
                        slope_left = slope_right;
                    }
                }
            } else {
                (weight_right, next_weight_left) = (third, third);
                (slope_right, next_slope_left) = (chord, chord);
            }
            if let Some((pt, _)) = before {
                let dx = weight_left * (t - pt);
                c.left[i] = (dx, dx * slope_left);
            }
            if let Some((nt, _)) = after {
                let dx = weight_right * (nt - t);
                c.right[i] = (dx, dx * slope_right);
            }
            (slope_left, weight_left) = (next_slope_left, next_weight_left);
        }
        c
    }

    /// The value at `time`: the first key's before it, the last key's
    /// after it.
    pub(crate) fn at(&self, time: f64) -> f64 {
        let (t, v) = (&self.times, &self.values);
        let k = t.partition_point(|&x| x <= time);
        if k == 0 {
            return v[0];
        }
        if k >= t.len() || t[k - 1] == time {
            return v[k - 1];
        }
        let span = t[k] - t[k - 1];
        let x = (time - t[k - 1]) / span;
        match self.ease[k - 1] {
            Ease::Hold => v[k - 1],
            Ease::HoldNext => v[k],
            Ease::Line => v[k - 1] + (v[k] - v[k - 1]) * x,
            Ease::Cubic => {
                // Where along the Bézier the time is `x`: its time runs
                // 0, x1, x2, 1 and rises, so halving finds it.
                let x1 = self.right[k - 1].0 / span;
                let x2 = 1.0 - self.left[k].0 / span;
                let time_at = |s: f64| {
                    let u = 1.0 - s;
                    3.0 * (u * u * s * x1 + u * s * s * x2) + s * s * s
                };
                let (mut lo, mut hi) = (0.0f64, 1.0f64);
                // Handles a third of the way each: the time is the place.
                let even = (x1 - 1.0 / 3.0).abs() < 1e-9 && (x2 - 2.0 / 3.0).abs() < 1e-9;
                if even {
                    (lo, hi) = (x, x);
                }
                for _ in 0..if even { 0 } else { 60 } {
                    let mid = (lo + hi) * 0.5;
                    if time_at(mid) < x {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let s = (lo + hi) * 0.5;
                let u = 1.0 - s;
                let (y0, y3) = (v[k - 1], v[k]);
                let (y1, y2) = (y0 + self.right[k - 1].1, y3 - self.left[k].1);
                u * u * u * y0 + 3.0 * (u * u * s * y1 + u * s * s * y2) + s * s * s * y3
            }
        }
    }
}

/// How much of each of a blend-shape channel's shapes there is at the
/// channel's `weight`: the shapes are whole at `keys` (rising), and
/// between two of them, or between none and the first, the channel is
/// part one and part the other; past the last it runs on as between
/// the last two.
pub(crate) fn shape_weights(weight: f64, keys: &[f64]) -> Vec<f64> {
    let n = keys.len();
    let mut out = vec![0.0; n];
    // The shapes below none (negative weights) end here.
    let below = keys.iter().take_while(|k| **k < 0.0).count();
    // `None` is no shape at all, at weight 0.
    let (mut prev, mut next): (Option<usize>, Option<usize>) = (None, None);
    if weight > 0.0 {
        prev = below.checked_sub(1);
        for (i, key) in keys.iter().enumerate().skip(below) {
            prev = next;
            next = Some(i);
            if *key > weight {
                break;
            }
        }
    } else {
        if below < n {
            prev = Some(below);
        }
        for i in (0..below).rev() {
            prev = next;
            next = Some(i);
            if keys[i] < weight {
                break;
            }
        }
    }
    let at = |k: Option<usize>| k.map_or(0.0, |i| keys[i]);
    let span = at(next) - at(prev);
    if span != 0.0 {
        let t = (weight - at(prev)) / span;
        if let Some(i) = prev {
            out[i] = 1.0 - t;
        }
        if let Some(i) = next {
            out[i] = t;
        }
    }
    out
}

/// A blend-shape channel's weight in percent: its `DeformPercent`.
pub(crate) fn deform_percent(channel: &&Node) -> f64 {
    channel
        .p70_num("DeformPercent")
        .or(channel
            .child("DeformPercent")
            .and_then(|c| c.props.first())
            .and_then(Prop::num))
        .filter(|w| w.is_finite())
        .unwrap_or(0.0)
}
