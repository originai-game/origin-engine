//! The Forge's 22 layer types (`TYPES` in its `engine.js`), each writing
//! its heat into the canvas, and the particle shapes and the 3x5 font
//! they draw with.

use super::engine::{
    C, DARK, DEG, Rng, TAU, angle_table, c01, clamp, ease, ease_name, fbm, fbm_p, hash, hash01,
    lerp, lookup, noise3, pair, smooth,
};
use super::jsmath::{self as m, hypot, max, min, round, truthy};
use super::json::Val;
use std::f64::consts::PI;

// ---------- 3x5 pixel font ----------

const FONT: &[(char, &str)] = &[
    ('A', "010101111101101"),
    ('B', "110101110101110"),
    ('C', "011100100100011"),
    ('D', "110101101101110"),
    ('E', "111100110100111"),
    ('F', "111100110100100"),
    ('G', "011100101101011"),
    ('H', "101101111101101"),
    ('I', "111010010010111"),
    ('J', "001001001101010"),
    ('K', "101101110101101"),
    ('L', "100100100100111"),
    ('M', "101111111101101"),
    ('N', "110101101101101"),
    ('O', "010101101101010"),
    ('P', "110101110100100"),
    ('Q', "010101101110011"),
    ('R', "110101110101101"),
    ('S', "011100010001110"),
    ('T', "111010010010010"),
    ('U', "101101101101111"),
    ('V', "101101101101010"),
    ('W', "101101111111101"),
    ('X', "101101010101101"),
    ('Y', "101101010010010"),
    ('Z', "111001010100111"),
    ('0', "111101101101111"),
    ('1', "010110010010111"),
    ('2', "110001010100111"),
    ('3', "110001010001110"),
    ('4', "101101111001001"),
    ('5', "111100110001110"),
    ('6', "011100111101111"),
    ('7', "111001010010010"),
    ('8', "111101111101111"),
    ('9', "111101111001110"),
    ('!', "010010010000010"),
    ('+', "000010111010000"),
    ('-', "000000111000000"),
    ('.', "000000000000010"),
    ('?', "110001010000010"),
    (':', "000010000010000"),
    ('/', "001001010100100"),
    (' ', "000000000000000"),
    ('>', "100010001010100"),
    ('<', "001010100010001"),
    ('_', "000000000000111"),
    (',', "000000000010100"),
    ('\'', "010010000000000"),
    ('(', "010100100100010"),
    (')', "010001001001010"),
    ('=', "000111000111000"),
    ('*', "000101010101000"),
];

// ---------- layer types ----------

fn burst(c: &mut C, l: &Val) {
    let (k, grow) = (c.lt, l.n("grow", 0.5));
    let r = lerp(
        l.n("r0", 0.08),
        l.n("r1", 0.8),
        ease(&ease_name(l, "outCubic"), k / grow),
    ) * (1.0 + l.n("drift", 0.12) * c01((k - grow) / (1.0 - grow)));
    let (freq, rough, z, s) = (
        l.n("freq", 2.4),
        l.n("rough", 0.55),
        k * l.n("evolve", 1.6),
        c.seed,
    );
    let cool_k = 1.0 - l.n("cool", 0.6) * ease("inOutSine", k);
    let er = l.n("erode", 1.0) * smooth(l.n("erodeStart", 0.3), 1.0, k);
    let (wob, hollow, gain) = (l.n("wobble", 0.3), l.n("hollow", 0.0), l.n("gain", 1.0));
    let tbl = angle_table(96, |a| {
        1.0 + wob * (fbm(m::cos(a) * 1.6 + 7.0, m::sin(a) * 1.6, z * 0.6, s + 5.0, 2) - 0.5) * 2.0
    });
    c.field_l(l, r * (1.0 + wob) * 1.3 + 0.05, |x, y| {
        let d0 = hypot(x, y);
        let d = d0 / (r * lookup(&tbl, m::atan2(y, x)));
        if d > 1.25 {
            return 0.0;
        }
        if er > 0.0
            && fbm(
                x * freq * 0.8 + 31.0,
                y * freq * 0.8,
                z * 0.7 + 3.0,
                s + 9.0,
                2,
            ) < er
        {
            return 0.0;
        }
        let n = fbm(x * freq, y * freq, z, s, 3);
        let mut v = (1.0 - d) * 1.35 + (n - 0.5) * rough * 2.0;
        if truthy(hollow) {
            v -= hollow * k * max(0.0, 1.0 - d * 1.6);
        }
        v * cool_k * gain
    });
}

fn ring(c: &mut C, l: &Val) {
    let k = c.lt;
    let r = lerp(
        l.n("r0", 0.1),
        l.n("r1", 0.95),
        ease(&ease_name(l, "outQuart"), k),
    );
    let th = lerp(l.n("th", 0.2), l.n("thEnd", 0.03), m::pow(k, 0.7));
    let (brk, freq, s) = (
        l.n("breakup", 0.7) * smooth(0.25, 1.0, k),
        l.n("freq", 3.0),
        c.seed,
    );
    let gain = l.n("gain", 1.0) * (1.0 - l.n("fade", 0.35) * k);
    let inner = l.n("inner", 0.0);
    let tbl = (brk > 0.0).then(|| {
        angle_table(128, |a| {
            fbm(m::cos(a) * freq, m::sin(a) * freq, k * 2.0, s, 2)
        })
    });
    c.field_l(l, r + th + 0.05, |x, y| {
        let d = hypot(x, y);
        let dd = (d - r).abs() / th;
        let mut v = 0.0;
        if dd <= 1.0
            && !tbl
                .as_ref()
                .is_some_and(|t| lookup(t, m::atan2(y, x)) < brk)
        {
            v = m::pow(1.0 - dd, 0.8) * gain * if d > r { 1.0 } else { 0.85 };
        }
        if truthy(inner) && d < r {
            v = max(v, inner * m::pow(d / r, 2.0) * gain);
        }
        v
    });
}

fn flame(c: &mut C, l: &Val) {
    let (w, hh, taper) = (l.n("w", 0.42), l.n("hgt", 1.2), l.n("taper", 1.1));
    let (freq, rough) = (l.n("freq", 2.6), l.n("rough", 0.75));
    let cyc = max(1.0, round(l.n("speed", 1.0)));
    let (p, s, hollow) = (4.0, c.seed, l.n("hollow", 0.0));
    let (flick, gain) = (l.n("flick", 0.35), l.n("gain", 1.05));
    let (mut env, mut hs) = (1.0, 1.0);
    if !c.looping {
        env = smooth(0.0, 0.14, c.lt) * (1.0 - smooth(0.6, 1.0, c.lt));
        hs = 0.35 + 0.65 * env;
    }
    let ph = c.ph;
    let scroll = ph * p * cyc;
    c.field_l(l, max(w * 1.5, hh) + 0.3, |x, y| {
        let ly = -y / (hh * hs);
        if ly < -0.3 || ly > 1.15 {
            return 0.0;
        }
        let sway = noise3(ly * 1.3, 3.7, ph * 2.0 * cyc, s + 3.0, 0.0, 0.0, 2.0 * cyc);
        let lx = x / w - sway * flick * max(0.0, ly) * 1.6;
        let prof = if ly < 0.0 {
            max(0.0, 1.0 - m::pow(ly / 0.3, 2.0)).sqrt()
        } else {
            m::pow(1.0 - min(ly, 1.0), taper)
        };
        let sx = 1.0 - lx.abs() / max(prof, 0.02);
        if sx < -0.6 {
            return 0.0;
        }
        let n = fbm_p(
            lx * freq * w * 1.6,
            ly * hh * freq - scroll,
            0.5,
            s,
            3,
            0.0,
            p,
            0.0,
        );
        let mut v = sx * 0.85 + (n - 0.5) * rough * 1.8 - max(0.0, ly) * 0.5 + 0.3;
        if truthy(hollow) {
            v -= hollow * max(0.0, 1.0 - lx.abs() / 0.62) * (1.0 - max(0.0, ly) * 0.55) * 1.4;
        }
        v * env * gain
    });
}

fn beam(c: &mut C, l: &Val) {
    let (len, th, cap, freq, wob) = (
        l.n("len", 1.8),
        l.n("th", 0.16),
        l.n("cap", 0.26),
        l.n("freq", 3.0),
        l.n("wob", 0.5),
    );
    let cyc = max(1.0, round(l.n("speed", 2.0)));
    let (p, s, gain) = (8.0, c.seed, l.n("gain", 1.05));
    let (mut ext, mut thick) = (1.0, th);
    if !c.looping {
        ext = ease("outCubic", c.lt / l.n("grow", 0.15));
        thick = th * (1.0 - smooth(0.72, 1.0, c.lt)) * (0.75 + 0.25 * smooth(0.0, 0.2, c.lt));
    }
    thick *= 1.0 + 0.1 * m::sin(TAU * c.ph * cyc * 2.0);
    let (lc, scroll, cap_r) = (len * ext, c.ph * p * cyc, cap * (thick / th));
    if thick < 0.005 {
        return;
    }
    c.field_l(l, len + cap + 0.2, |a, b| {
        let mut v = 0.0;
        if a >= -0.05 && a <= lc {
            let n = fbm_p(a * freq - scroll, b * freq * 0.6, 0.3, s, 2, p, 0.0, 0.0);
            let t = thick * (1.0 + wob * (n - 0.5) * 1.6) * (1.0 - 0.25 * (a / len));
            let tip = c01((lc - a) / 0.12);
            let q = b.abs() / max(t * tip.sqrt(), 1e-3);
            if q < 1.0 {
                v = (1.0 - m::pow(q, 1.6)) * 1.1 + (n - 0.5) * 0.35;
            }
        }
        let d = hypot(a, b) / max(cap_r, 1e-3);
        if d < 1.0 {
            v = max(v, (1.0 - d) * 1.25);
        }
        v * gain
    });
}

fn slash(c: &mut C, l: &Val) {
    let (r, th) = (l.n("r", 0.72), l.n("th", 0.2));
    let a0 = l.n("a0", -160.0) * DEG;
    let span = (l.n("a1", 40.0) - l.n("a0", -160.0)) * DEG;
    let (sweep, trail, k) = (l.n("sweep", 0.3), l.n("trail", 0.9), c.lt);
    let head = ease("outCubic", k / sweep);
    let after = smooth(sweep, 1.0, k);
    let tail = max(0.0, head - trail * (1.0 - after));
    let thin = m::pow(1.0 - after, 0.6);
    let (gain, s, er, aspan) = (l.n("gain", 1.0), c.seed, l.n("erode", 0.6), span.abs());
    c.field_l(l, r + 0.15, |x, y| {
        let rad = hypot(x, y);
        if rad > r + 0.06 || rad < r - 2.4 * th {
            return 0.0;
        }
        let mut a = m::atan2(y, x) - a0;
        if span < 0.0 {
            a = -a;
        }
        a = ((a % TAU) + TAU) % TAU;
        let p = a / aspan;
        if p < tail || p > head {
            return 0.0;
        }
        let q = (p - tail) / max(head - tail, 1e-4);
        let w = 2.0 * th * m::pow(m::sin(PI * m::pow(q, 1.25)), 0.7) * thin;
        if w <= 0.008 {
            return 0.0;
        }
        let dr = r - rad;
        if dr < -0.02 || dr > w {
            return 0.0;
        }
        if er > 0.0 && after > 0.0 && fbm(x * 5.0, y * 5.0, k * 2.0, s, 2) < er * (1.0 - q) * after
        {
            return 0.0;
        }
        (0.25 + 0.85 * (1.0 - dr / w)) * (0.55 + 0.45 * q) * gain
    });
}

fn orb(c: &mut C, l: &Val) {
    let (r, freq, pulse) = (l.n("r", 0.32), l.n("freq", 2.8), l.n("pulse", 0.08));
    let spin = round(l.n("spin", 1.0));
    let (s, gain, rim) = (c.seed, l.n("gain", 1.0), l.n("rim", 0.35));
    let mut env = 1.0;
    if !c.looping {
        env = ease("outBack", c.lt / l.n("grow", 0.2)) * (1.0 - smooth(0.75, 1.0, c.lt));
    }
    let rr = r * env * (1.0 + pulse * m::sin(TAU * c.ph * max(1.0, round(l.n("pulseRate", 2.0)))));
    if rr <= 0.01 {
        return;
    }
    let ang = TAU * c.ph * spin;
    let (ca, sa) = (m::cos(ang), m::sin(ang));
    c.field_l(l, rr * (1.0 + rim) + 0.05, |x, y| {
        let d = hypot(x, y) / rr;
        if d > 1.0 + rim {
            return 0.0;
        }
        let (rx, ry) = (x * ca - y * sa, x * sa + y * ca);
        let n = fbm(
            (rx / r) * freq * 0.5 + 5.0,
            (ry / r) * freq * 0.5,
            0.7,
            s,
            3,
        );
        if d <= 1.0 {
            return ((1.0 - d * d) * 1.05 + (n - 0.5) * 0.7 + 0.15) * gain;
        }
        (1.0 - (d - 1.0) / rim) * 0.35 * if n > 0.45 { 1.0 } else { 0.45 } * gain
    });
}

fn comet(c: &mut C, l: &Val) {
    let (r, len, freq, rough) = (
        l.n("r", 0.2),
        l.n("len", 1.1),
        l.n("freq", 3.0),
        l.n("rough", 0.6),
    );
    let cyc = max(1.0, round(l.n("speed", 2.0)));
    let (p, s, gain) = (8.0, c.seed, l.n("gain", 1.0));
    let (mut ax, mut ay, mut angle, mut vis) =
        (l.n("x", 0.0), l.n("y", 0.0), l.n("angle", 0.0), 1.0);
    if l.t("from") {
        let from = |i: usize| match l.get("from") {
            Some(Val::Arr(a)) => a.get(i).map_or(f64::NAN, Val::num),
            _ => f64::NAN,
        };
        let until = l.n("until", 0.45);
        let e = ease("inQuad", c.lt / until);
        if c.lt > until {
            return;
        }
        ax = lerp(from(0), l.n("x", 0.0), e);
        ay = lerp(from(1), l.n("y", 0.0), e);
        angle = m::atan2(l.n("y", 0.0) - from(1), l.n("x", 0.0) - from(0)) / DEG;
        vis = 1.0;
    } else if !c.looping {
        vis = smooth(0.0, 0.1, c.lt) * (1.0 - smooth(0.8, 1.0, c.lt));
    }
    let scroll = c.ph * p * cyc;
    c.field(l.n("squash", 1.0), len + r + 0.2, ax, ay, angle, |a, b| {
        let dh = hypot(a, b) / r;
        let mut v = if dh < 1.3 {
            (1.0 - dh) * 1.3 + 0.12
        } else {
            0.0
        };
        if a < 0.05 {
            let t = -a / len;
            if t <= 1.0 {
                let n = fbm_p(a * freq + scroll, b * freq, 0.2, s, 3, p, 0.0, 0.0);
                let wid = r * m::pow(1.0 - t, 0.7) * (0.85 + 1.2 * (n - 0.5));
                let q = b.abs() / max(wid, 1e-3);
                if q < 1.0 {
                    v = max(v, (1.0 - q) * (1.0 - t) * 1.15 + (n - 0.5) * rough);
                }
            }
        }
        v * gain * vis
    });
}

fn vortex(c: &mut C, l: &Val) {
    let (r, r_in) = (l.n("r", 0.85), l.n("rIn", 0.18));
    let arms = max(1.0, round(l.n("arms", 3.0)));
    let twist = l.n("twist", 2.2);
    let sp = round(l.n("spin", 1.0));
    let spin = if truthy(sp) { sp } else { 1.0 } * l.n("dir", 1.0);
    let (freq, s, gain) = (l.n("freq", 2.2), c.seed, l.n("gain", 1.0));
    let dark = l.s("core") == Some("dark");
    let mut env = 1.0;
    if !c.looping {
        env = ease("outBack", c.lt / 0.25) * (1.0 - smooth(0.75, 1.0, c.lt));
    }
    let rr = r * env;
    if rr < 0.02 {
        return;
    }
    let rot = TAU * c.ph * spin;
    let (cr, sr) = (m::cos(rot), m::sin(rot));
    c.field_l(l, rr + 0.05, |x, y| {
        let rad = hypot(x, y) / rr;
        if rad > 1.0 {
            return 0.0;
        }
        if rad < r_in {
            return if dark {
                DARK
            } else {
                (1.0 - rad / r_in) * 0.5 + 0.9
            };
        }
        let th = m::atan2(y, x) - rot;
        let stripe = 0.5 + 0.5 * m::cos(TAU * (arms * th / TAU + twist * rad));
        let (rx, ry) = (x * cr + y * sr, -x * sr + y * cr);
        let n = fbm((rx / r) * freq * 1.5, (ry / r) * freq * 1.5, 0.4, s, 2);
        let mm = smooth(r_in, r_in + 0.15, rad) * m::pow(1.0 - rad, 0.6);
        let edge = if dark {
            (1.0 - smooth(r_in, r_in + 0.1, rad)) * 1.0
        } else {
            0.0
        };
        max((0.25 + 0.95 * stripe * (0.5 + n)) * mm * gain, edge * gain)
    });
}

fn pillar(c: &mut C, l: &Val) {
    let (w, hh, freq, rough) = (
        l.n("w", 0.28),
        l.n("hgt", 1.8),
        l.n("freq", 3.0),
        l.n("rough", 0.5),
    );
    let cyc = max(1.0, round(l.n("speed", 2.0)));
    let (p, s, gain) = (8.0, c.seed, l.n("gain", 1.0));
    let (mut grow, mut wid) = (1.0, 1.0);
    if !c.looping {
        grow = ease("outCubic", c.lt / l.n("grow", 0.18));
        wid = (1.0 - smooth(0.55, 1.0, c.lt)) * (0.6 + 0.4 * smooth(0.0, 0.1, c.lt));
    }
    let (hc, wc, scroll) = (max(hh * grow, 0.02), w * wid, c.ph * p * cyc);
    if wc < 0.01 {
        return;
    }
    c.field_l(l, hh + 0.3, |x, y| {
        let ly = -y / hc;
        let mut v = 0.0;
        if (0.0..=1.0).contains(&ly) {
            let n = fbm_p(
                x * freq * 1.5,
                ly * hc * freq - scroll,
                0.6,
                s,
                3,
                0.0,
                p,
                0.0,
            );
            let wv = wc * (1.0 + 0.35 * (n - 0.5)) * (1.0 - 0.15 * ly);
            let q = x.abs() / wv;
            if q < 1.0 {
                v = (1.0 - q * q) * (1.0 - m::pow(ly, 3.0)) * 1.15 + (n - 0.5) * rough;
            }
        }
        let bd = hypot(x / (wc * 1.8), y / (wc * 0.55));
        if bd < 1.0 {
            v = max(v, (1.0 - bd) * 1.1);
        }
        v * gain
    });
}

fn rays(c: &mut C, l: &Val) {
    let n = max(2.0, round(l.n("count", 10.0)));
    let (r, wdt, jit) = (l.n("r", 0.9), l.n("width", 0.2), l.n("jitter", 0.45));
    let (s, gain, inner) = (c.seed, l.n("gain", 1.0), l.n("inner", 0.08));
    let twk = max(1.0, round(l.n("twinkle", 1.0)));
    let (env, rot) = if c.looping {
        (1.0, round(l.n("spin", 0.0)) * c.ph)
    } else {
        (
            ease("outCubic", c.lt / 0.25) * (1.0 - smooth(0.6, 1.0, c.lt)),
            l.n("spin", 0.08) * c.lt,
        )
    };
    let rc = r * if c.looping { 1.0 } else { 0.5 + 0.5 * env };
    let ph = c.ph;
    c.field_l(l, r + 0.1, |x, y| {
        let rad = hypot(x, y);
        let a = ((((m::atan2(y, x) / TAU + rot) % 1.0) + 1.0) % 1.0) * n;
        let i = a.floor();
        let f = a - i - 0.5;
        let len = rc
            * (1.0 - jit * hash01(i, 0.0, 0.0, s))
            * (1.0 + 0.18 * m::sin(TAU * (ph * twk + hash01(i, 1.0, 0.0, s))));
        if rad > len || rad < inner {
            return 0.0;
        }
        let tw = wdt * (1.0 - rad / len) + 0.02;
        if f.abs() > tw {
            return 0.0;
        }
        (1.0 - rad / len) * 1.15 * (1.0 - (f.abs() / tw) * 0.5) * env * gain
    });
}

fn glyph(c: &mut C, l: &Val) {
    let r = l.n("r", 0.8) * c.s;
    let sq = l.n("squash", 1.0);
    let cx = c.w / 2.0 + l.n("x", 0.0) * c.s - 0.5;
    let cy = c.h / 2.0 + l.n("y", 0.0) * c.s - 0.5;
    let draw_t = if c.looping {
        1.0
    } else {
        ease("outQuad", c.lt / l.n("draw", 0.35))
    };
    let env = if c.looping {
        1.0
    } else {
        1.0 - smooth(0.75, 1.0, c.lt)
    };
    let rot = if c.looping {
        TAU * c.ph * round(l.n("spin", 1.0))
    } else {
        TAU * c.lt * l.n("spin", 0.25)
    };
    let gain = l.n("gain", 1.0) * env * (0.92 + 0.08 * hash01(c.f, 9.0, 0.0, c.seed));
    if gain <= 0.02 {
        return;
    }
    let rings: Vec<f64> = match l.opt("rings") {
        Some(Val::Arr(a)) => a.iter().map(Val::num).collect(),
        Some(_) => Vec::new(),
        None => vec![1.0, 0.84, 0.5],
    };
    let pt = |c: &mut C, a: f64, rr: f64, v: f64| {
        c.plot(
            round(cx + m::cos(a) * rr),
            round(cy + m::sin(a) * rr * sq),
            v,
        );
    };
    for (i, &rr) in rings.iter().enumerate() {
        let rp = r * rr;
        let n = (TAU * rp * 1.6).ceil() + 8.0;
        let dir = if i % 2 == 1 { -1.0 } else { 1.0 };
        let lim = n * draw_t;
        let mut k = 0.0;
        while k < lim {
            pt(
                c,
                -PI / 2.0 + dir * (k / n) * TAU + dir * rot,
                rp,
                gain * if i == 0 { 0.82 } else { 0.66 },
            );
            k += 1.0;
        }
    }
    if rings.len() > 1 {
        let t = l.n("ticks", 24.0);
        let (r0, r1) = (r * rings[1], r * rings[0]);
        let mut k = 0.0;
        while k < t * draw_t {
            let a = -PI / 2.0 - rot + (k / t) * TAU;
            let rm = r0 + (r1 - r0) * 0.6;
            c.line1(
                round(cx + m::cos(a) * r0),
                round(cy + m::sin(a) * r0 * sq),
                round(cx + m::cos(a) * rm),
                round(cy + m::sin(a) * rm * sq),
                gain * 0.58,
            );
            k += 1.0;
        }
    }
    let p = round(l.n("star", 5.0));
    let step = round(l.n("step", 2.0));
    if p >= 3.0 {
        let rs = r * rings.last().copied().unwrap_or(0.5);
        let vtx = |j: f64| {
            let a = -PI / 2.0 + (j / p) * TAU + rot * 0.5;
            (round(cx + m::cos(a) * rs), round(cy + m::sin(a) * rs * sq))
        };
        let edges = (p * draw_t).ceil();
        let mut j = 0.0;
        while j < edges {
            let ((x0, y0), (x1, y1)) = (vtx(j * step), vtx((j + 1.0) * step));
            c.line1(x0, y0, x1, y1, gain * 0.86);
            j += 1.0;
        }
    }
    let runes = round(l.n("runes", 8.0));
    if truthy(runes) && rings.len() > 2 {
        let rm = r * (rings[1] + rings[2]) / 2.0;
        let mut j = 0.0;
        while j < runes * draw_t {
            let a = -PI / 2.0 + (j / runes) * TAU - rot;
            let gx = round(cx + m::cos(a) * rm) - 1.0;
            let gy = round(cy + m::sin(a) * rm * sq) - 1.0;
            let mut bits = hash(j, 77.0, 0.0, c.seed) & 511;
            if bits & 0x10 == 0 {
                bits |= 0x10 | 0x101;
            }
            for b in 0..9u32 {
                if bits & (1 << b) != 0 {
                    c.plot(gx + f64::from(b % 3), gy + f64::from(b / 3), gain * 1.02);
                }
            }
            j += 1.0;
        }
    }
    if l.t("fill") {
        let fill = l.n("fill", 0.0);
        let r2 = r * r;
        let mut y = (cy - r * sq).floor();
        while y <= cy + r * sq {
            let mut x = (cx - r).floor();
            while x <= cx + r {
                let (dx, dy) = (x - cx, (y - cy) / sq);
                if dx * dx + dy * dy < r2 {
                    c.plot(x, y, fill * gain);
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }
}

fn puffs(c: &mut C, l: &Val) {
    let n = round(l.n("count", 7.0));
    let s = c.seed;
    let mut rand = Rng::new(s);
    let (spread, rise, r0, r1) = (
        l.n("spread", 0.4),
        l.n("rise", 0.5),
        l.n("r0", 0.12),
        l.n("r1", 0.3),
    );
    let [lf_a, lf_b] = pair(l, "life", [0.55, 1.0]);
    let [sp_a, sp_b] = pair(l, "speed", [0.0, 0.0]);
    let (emit, dir, cone) = (
        l.n("emit", 0.25),
        l.n("dir", -90.0) * DEG,
        l.n("cone", 0.0) * DEG,
    );
    let (freq, gain) = (l.n("freq", 3.2), l.n("gain", 0.6));
    let (diss, decay) = (l.n("dissolve", 1.0), l.n("decay", 0.5));
    let (lx, ly, drift, flat) = (
        l.n("x", 0.0),
        l.n("y", 0.0),
        l.n("drift", 0.3),
        l.n("flat", 0.6),
    );
    let mut i = 0.0;
    while i < n {
        let q = rand.q7();
        let life = lerp(lf_a, lf_b, q[0]);
        let birth = if c.looping {
            (i + q[1] * 0.5) / n
        } else {
            q[1] * emit
        };
        let age = if c.looping {
            ((((c.ph - birth) % 1.0) + 1.0) % 1.0) / life
        } else {
            (c.lt - birth) / life
        };
        i += 1.0;
        if age < 0.0 || age > 1.0 {
            continue;
        }
        let e = ease("outCubic", age);
        let ang = dir + (q[2] - 0.5) * cone;
        let sp = lerp(sp_a, sp_b, q[3]);
        let cxu =
            lx + (q[4] - 0.5) * 2.0 * spread + m::cos(ang) * sp * e + (q[5] - 0.5) * drift * age;
        let cyu = ly + (q[6] - 0.5) * spread * flat + m::sin(ang) * sp * e - rise * e;
        let rr = (r0 + (r1 - r0) * e) * (0.8 + 0.4 * q[0]);
        let thr = m::pow(age, 1.4) * diss;
        let hm = gain * (1.0 - decay * age);
        let (pcx, pcy, pr) = (c.w / 2.0 + cxu * c.s, c.h / 2.0 + cyu * c.s, rr * c.s);
        let ii = i - 1.0;
        let mut y = max(0.0, (pcy - pr).floor());
        while y <= min(c.h - 1.0, (pcy + pr).ceil()) {
            let mut x = max(0.0, (pcx - pr).floor());
            while x <= min(c.w - 1.0, (pcx + pr).ceil()) {
                let d = hypot(x + 0.5 - pcx, y + 0.5 - pcy) / pr;
                if d <= 1.0 {
                    let nn = fbm(
                        (x / c.s) * freq + ii * 7.1,
                        (y / c.s) * freq,
                        age * 1.5,
                        s,
                        2,
                    );
                    if nn >= thr {
                        let v = ((1.0 - d) * 0.8 + (nn - 0.5) * 0.8 + 0.3) * hm;
                        let j = y as usize * c.wi + x as usize;
                        if v > f64::from(c.heat[j]) {
                            c.heat[j] = v as f32;
                        }
                    }
                }
                x += 1.0;
            }
            y += 1.0;
        }
    }
}

/// A particle's shape, drawn in pixel space.
#[allow(clippy::too_many_arguments)]
fn shape(
    c: &mut C,
    kind: &str,
    x: f64,
    y: f64,
    size: f64,
    v: f64,
    vx: f64,
    vy: f64,
    age: f64,
    r: f64,
) {
    let (xx, yy) = (round(x), round(y));
    let range = |a: f64, b: f64| {
        let mut out = Vec::new();
        let mut k = a;
        while k <= b {
            out.push(k);
            k += 1.0;
        }
        out
    };
    match kind {
        "square" => {
            for j in range(0.0, size - 1.0) {
                for i in range(0.0, size - 1.0) {
                    c.plot(xx + i, yy + j, v);
                }
            }
        }
        "plus" | "cross" => {
            let a = max(1.0, size);
            c.plot(xx, yy, v);
            for k in range(1.0, a) {
                let vv = if kind == "plus" {
                    v * (1.0 - (k / (a + 1.0)) * 0.5)
                } else {
                    v
                };
                c.plot(xx + k, yy, vv);
                c.plot(xx - k, yy, vv);
                c.plot(xx, yy + k, vv);
                c.plot(xx, yy - k, vv);
            }
            if kind == "cross" && size >= 3.0 {
                c.plot(xx + 1.0, yy + 1.0, v * 0.7);
                c.plot(xx - 1.0, yy - 1.0, v * 0.7);
                c.plot(xx + 1.0, yy - 1.0, v * 0.7);
                c.plot(xx - 1.0, yy + 1.0, v * 0.7);
            }
        }
        "star" => {
            let a = max(1.0, round(size * m::sin(PI * c01(age))));
            c.plot(xx, yy, v * 1.1);
            for k in range(1.0, a) {
                let vv = v * (1.0 - k / (a + 1.0));
                c.plot(xx + k, yy, vv);
                c.plot(xx - k, yy, vv);
                c.plot(xx, yy + k, vv);
                c.plot(xx, yy - k, vv);
            }
        }
        "diamond" => {
            for j in range(-size, size) {
                for i in range(-size, size) {
                    if i.abs() + j.abs() <= size {
                        c.plot(
                            xx + i,
                            yy + j,
                            v * (1.0 - (i.abs() + j.abs()) / (size + 1.0) * 0.4),
                        );
                    }
                }
            }
        }
        "ring" | "bubble" => {
            let rr = max(1.0, size);
            for k in 0..24 {
                let a = (f64::from(k) / 24.0) * TAU;
                c.plot(round(xx + m::cos(a) * rr), round(yy + m::sin(a) * rr), v);
            }
            if kind == "bubble" && rr >= 2.0 {
                c.plot(xx - 1.0, yy - 1.0, v * 1.15);
            }
        }
        "streak" | "drop" => {
            let sp = hypot(vx, vy);
            let sp = if truthy(sp) { sp } else { 1.0 };
            let len = if kind == "drop" {
                1.0
            } else {
                max(1.0, round(size * sp * 0.6))
            };
            let (x1, y1) = (round(xx - (vx / sp) * len), round(yy - (vy / sp) * len));
            c.line(xx, yy, x1, y1, v, v * 0.45, 1.0);
        }
        "shard" => {
            let a = r * PI + age * 6.0;
            let l = max(1.0, size);
            c.line(
                xx,
                yy,
                round(xx + m::cos(a) * l),
                round(yy + m::sin(a) * l),
                v,
                v * 0.7,
                1.0,
            );
        }
        "leaf" | "petal" => {
            let flip = truthy((age * 8.0 + r * 4.0).floor() % 2.0);
            c.plot(xx, yy, v);
            c.plot(
                xx + if flip { 1.0 } else { -1.0 },
                yy + if flip { 0.0 } else { 1.0 },
                v * 0.8,
            );
            if size > 1.0 {
                c.plot(xx + if flip { 1.0 } else { -1.0 }, yy + 1.0, v * 0.6);
            }
        }
        "chevron" => {
            let a = max(2.0, size);
            for k in range(0.0, a) {
                c.plot(xx - k, yy + k, v);
                c.plot(xx + k, yy + k, v);
            }
        }
        "coin" => {
            let wd = max(0.0, round((m::cos(age * TAU * 2.0 + r * TAU)).abs() * size));
            for j in range(-size, size) {
                for i in range(-wd, wd) {
                    if (i * i) / m::pow(wd + 0.5, 2.0) + (j * j) / m::pow(size + 0.5, 2.0) <= 1.0 {
                        c.plot(
                            xx + i,
                            yy + j,
                            v * if i < 0.0 && j < 0.0 { 1.1 } else { 0.8 },
                        );
                    }
                }
            }
        }
        "chunk" => {
            let rot = truthy((age * 10.0 + r * 4.0).floor() % 2.0);
            for j in range(0.0, size - 1.0) {
                for i in range(0.0, size - 1.0) {
                    if rot && size > 2.0 && (i + j == 0.0 || i + j == 2.0 * size - 2.0) {
                        continue;
                    }
                    c.plot(
                        xx + i,
                        yy + j,
                        v * if i == 0.0 && j == 0.0 { 1.15 } else { 0.8 },
                    );
                }
            }
        }
        "circle" => c.disc(xx, yy, max(0.5, size / 2.0), v),
        _ => {
            if size <= 1.0 {
                c.plot(xx, yy, v);
            } else {
                c.disc(xx, yy, size / 2.0, v);
            }
        }
    }
}

fn particles(c: &mut C, l: &Val) {
    let n = round(l.n("count", 16.0));
    let mut rand = Rng::new(c.seed);
    let mode = match l.get("mode") {
        Some(v) if v.truthy() => v.js_string(),
        _ => "burst".into(),
    };
    let [sp_a, sp_b] = pair(l, "speed", [0.8, 1.6]);
    let [lf_a, lf_b] = pair(l, "life", [0.5, 1.0]);
    let [sz_a, sz_b] = pair(l, "size", [1.0, 1.0]);
    let [h0, h1] = pair(l, "heat", [1.0, 0.35]);
    let (grav, drag, emit) = (l.n("gravity", 0.0), l.n("drag", 1.5), l.n("emit", 0.0));
    let (dir, spread, area) = (
        l.n("dir", 0.0) * DEG,
        l.n("spread", 360.0) * DEG,
        l.n("area", 0.05),
    );
    let kind = match l.get("shape") {
        Some(v) if v.truthy() => v.js_string(),
        _ => "dot".into(),
    };
    let (gain, sq) = (l.n("gain", 1.0), l.n("squash", 1.0));
    let (ax, ay, wu, hu) = (l.n("x", 0.0), l.n("y", 0.0), c.w / c.s, c.h / c.s);
    let ground = l.opt("ground").map(Val::num);
    let twinkle = l.t("twinkle").then(|| l.n("twinkle", 0.0));
    let fade_in = l.t("fadeIn").then(|| l.n("fadeIn", 0.0));
    let mut i = 0.0;
    while i < n {
        let q = rand.q7();
        let life = lerp(lf_a, lf_b, q[0]);
        let sp = lerp(sp_a, sp_b, q[1]);
        let (age, tau);
        if c.looping {
            let ph0 = (i + q[2] * l.n("jitter", 0.8)) / n;
            age = ((((c.ph - ph0) % 1.0) + 1.0) % 1.0) / life;
            tau = age * life;
        } else {
            tau = c.lt - q[2] * emit;
            age = tau / life;
        }
        i += 1.0;
        if age < 0.0 || age > 1.0 {
            continue;
        }
        let ang = dir + (q[3] - 0.5) * spread;
        let (ca, sa) = (m::cos(ang), m::sin(ang));
        let (s_ang, s_r) = (q[4] * TAU, q[5].sqrt() * area);
        let x0 = ax + m::cos(s_ang) * s_r * l.n("areaX", 1.0);
        let y0 = ay + m::sin(s_ang) * s_r * sq * l.n("areaY", 1.0);
        let (x, mut y, vx, mut vy);
        match mode.as_str() {
            "rise" => {
                let wig = l.n("wiggle", 0.06) * m::sin(TAU * (age * l.n("wiggleRate", 1.5) + q[6]));
                x = x0 + wig + ca * sp * tau * 0.2;
                y = y0 - sp * tau;
                vx = 0.0;
                vy = -sp;
            }
            "fall" => {
                let (xs, ys, dist) = ((q[4] - 0.5) * wu * 1.15, -hu / 2.0 - 0.08, hu + 0.2);
                let drift = l.n("driftX", 0.15) * (0.5 + q[1]);
                let wig = l.n("wiggle", 0.0) * m::sin(TAU * (age * l.n("wiggleRate", 2.0) + q[6]));
                x = xs + drift * age + wig;
                y = ys + age * dist * l.n("fallScale", 1.0) * (0.85 + 0.3 * q[5]);
                vx = drift;
                vy = dist;
            }
            "ambient" => {
                let xs = (q[4] - 0.5) * wu * l.n("spanX", 1.0);
                let ys = (q[5] - 0.5) * hu * l.n("spanY", 1.0) + ay;
                let wx = l.n("wiggle", 0.08) * m::sin(TAU * (age + q[6]));
                let wy = l.n("wiggle", 0.08) * m::cos(TAU * (age * 0.7 + q[0]));
                x = xs + wx;
                y = ys + wy - sp * tau * 0.25;
                vx = 0.0;
                vy = 0.0;
            }
            "orbit" => {
                let rr = l.n("orbitR", 0.6) * lerp(0.75, 1.1, q[5]);
                let t = round(l.n("turns", 1.0));
                let turns = if truthy(t) { t } else { 1.0 };
                let sign = if l.t("both") && q[6] < 0.5 { -1.0 } else { 1.0 };
                let a = q[3] * TAU + sign * TAU * turns * if c.looping { c.ph } else { c.lt };
                x = ax + m::cos(a) * rr;
                y = ay + m::sin(a) * rr * sq;
                vx = -m::sin(a) * sign;
                vy = m::cos(a) * sq * sign;
            }
            "implode" => {
                let rr = l.n("implodeR", 0.9) * lerp(0.6, 1.0, q[5]);
                let e = ease("inCubic", age);
                let a = q[3] * TAU + l.n("swirl", 0.6) * e;
                x = ax + m::cos(a) * rr * (1.0 - e);
                y = ay + m::sin(a) * rr * (1.0 - e) * sq;
                vx = -m::cos(a) * 2.0;
                vy = -m::sin(a) * 2.0 * sq;
            }
            _ => {
                let d = if drag > 0.0 {
                    (1.0 - m::exp(-drag * tau)) / drag
                } else {
                    tau
                };
                let dv = m::exp(-drag * tau);
                x = x0 + ca * sp * d;
                y = y0 + sa * sp * d * sq + 0.5 * grav * tau * tau;
                vx = ca * sp * dv;
                vy = sa * sp * dv * sq + grav * tau;
                if let Some(g) = ground
                    && y > g
                {
                    y = g;
                    vy = 0.0;
                }
            }
        }
        let mut v = lerp(h0, h1, age) * gain;
        if let Some(tw) = twinkle {
            v *= 0.55 + 0.45 * m::sin(TAU * (age * tw + q[6])).abs();
        }
        if let Some(fi) = fade_in {
            v *= smooth(0.0, fi, age);
        }
        let size = max(1.0, round(lerp(sz_a, sz_b, age)));
        let (sx, sy) = (c.w / 2.0 + x * c.s - 0.5, c.h / 2.0 + y * c.s - 0.5);
        shape(c, &kind, sx, sy, size, v, vx, vy, age, q[6]);
    }
}

fn bolt(c: &mut C, l: &Val) {
    if l.t("flicker") && hash01(c.f, 3.0, 0.0, c.seed) < l.n("flicker", 0.0) {
        return;
    }
    let hold = max(1.0, round(l.n("hold", 2.0)));
    let mut rand = Rng::new(c.seed + (c.f / hold).floor() * 7919.0);
    let env = if c.looping {
        1.0
    } else {
        1.0 - smooth(l.n("fadeStart", 0.5), 1.0, c.lt)
    };
    let gain = l.n("gain", 1.0) * env;
    if gain < 0.05 {
        return;
    }
    let (ax, ay) = (l.n("x", 0.0), l.n("y", 0.0));
    let mode = match l.get("mode") {
        Some(v) if v.truthy() => v.js_string(),
        _ => "radial".into(),
    };
    let count = round(l.n("count", 4.0));
    let (jag, depth, branch, thick) = (
        l.n("jag", 0.35),
        round(l.n("depth", 5.0)),
        l.n("branch", 0.3),
        l.n("thick", 1.0),
    );
    let mut paths: Vec<[f64; 4]> = Vec::new();
    let mut k = 0.0;
    if mode == "radial" {
        let reach = l.n("len", 0.9)
            * if c.looping {
                1.0
            } else {
                0.4 + 0.6 * ease("outCubic", c.lt / 0.2)
            };
        while k < count {
            let a = (k / count + (rand.next() - 0.5) * 0.7 / count) * TAU + l.n("angle", 0.0) * DEG;
            let ln = reach * lerp(0.55, 1.0, rand.next());
            paths.push([
                ax,
                ay,
                ax + m::cos(a) * ln,
                ay + m::sin(a) * ln * l.n("squash", 1.0),
            ]);
            k += 1.0;
        }
    } else if mode == "strike" {
        let top = -c.h / c.s / 2.0 - 0.05;
        let reach = ease("outExpo", c.lt / 0.12);
        while k < count {
            let sx = ax + (rand.next() - 0.5) * l.n("spread", 0.35);
            paths.push([sx, top, lerp(sx, ax, reach), lerp(top, ay, reach)]);
            k += 1.0;
        }
    } else {
        while k < count {
            paths.push([
                l.n("x0", -0.8),
                l.n("y0", 0.0),
                l.n("x1", 0.8),
                l.n("y1", 0.0),
            ]);
            k += 1.0;
        }
    }
    let draw_path = |c: &mut C, pts: &[[f64; 2]], v: f64, th: f64| {
        for w in pts.windows(2) {
            let (x0, y0, x1, y1) = (c.px(w[0][0]), c.py(w[0][1]), c.px(w[1][0]), c.py(w[1][1]));
            c.line(x0, y0, x1, y1, v, v, th);
        }
    };
    let displace = |rand: &mut Rng, a: [f64; 2], b: [f64; 2], dpt: f64, j: f64| {
        let mut pts = vec![a, b];
        let mut d = 0.0;
        while d < dpt {
            let mut nx = vec![pts[0]];
            for w in pts.windows(2) {
                let (p, q) = (w[0], w[1]);
                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                let len = hypot(dx, dy);
                let len = if truthy(len) { len } else { 1.0 };
                let off = (rand.next() - 0.5) * j * len;
                nx.push([
                    (p[0] + q[0]) / 2.0 - (dy / len) * off,
                    (p[1] + q[1]) / 2.0 + (dx / len) * off,
                ]);
                nx.push(q);
            }
            pts = nx;
            d += 1.0;
        }
        pts
    };
    for [x0, y0, x1, y1] in paths {
        let pts = displace(&mut rand, [x0, y0], [x1, y1], depth, jag);
        draw_path(c, &pts, gain, thick);
        let mut k = 2;
        while k + 2 < pts.len() {
            if rand.next() > branch {
                k += 3;
                continue;
            }
            let (p, q) = (pts[k], pts[(pts.len() - 1).min(k + 2)]);
            let sign = if rand.next() < 0.5 { -1.0 } else { 1.0 };
            let a = m::atan2(q[1] - p[1], q[0] - p[0]) + sign * lerp(0.35, 0.9, rand.next());
            let ln = hypot(x1 - x0, y1 - y0) * lerp(0.15, 0.35, rand.next());
            let b = displace(
                &mut rand,
                p,
                [p[0] + m::cos(a) * ln, p[1] + m::sin(a) * ln],
                3.0,
                jag,
            );
            draw_path(c, &b, gain * 0.72, 1.0);
            k += 3;
        }
    }
}

fn cracks(c: &mut C, l: &Val) {
    let n = round(l.n("count", 6.0));
    let mut rand = Rng::new(c.seed);
    let (len, sq) = (l.n("len", 0.9) * c.s, l.n("squash", 0.4));
    let grow = ease("outCubic", c.lt / l.n("grow", 0.2));
    let env = 1.0 - smooth(l.n("fadeStart", 0.55), 1.0, c.lt);
    let gain = l.n("gain", 1.0) * env;
    let cx = c.w / 2.0 + l.n("x", 0.0) * c.s - 0.5;
    let cy = c.h / 2.0 + l.n("y", 0.0) * c.s - 0.5;
    if gain < 0.05 {
        return;
    }
    #[allow(clippy::too_many_arguments)]
    fn walk(
        c: &mut C,
        rand: &mut Rng,
        sq: f64,
        mut x: f64,
        mut y: f64,
        mut a: f64,
        steps: f64,
        v: f64,
    ) {
        let (mut px0, mut py0) = (round(x), round(y));
        let mut k = 0.0;
        while k < steps {
            a += (rand.next() - 0.5) * 0.9;
            x += m::cos(a) * 1.3;
            y += m::sin(a) * 1.3 * sq;
            let (nx, ny) = (round(x), round(y));
            c.line1(px0, py0, nx, ny, v * (1.0 - (k / steps) * 0.55));
            (px0, py0) = (nx, ny);
            if rand.next() < 0.08 && steps - k > 4.0 {
                let turn = if rand.next() < 0.5 { -0.8 } else { 0.8 };
                walk(
                    c,
                    rand,
                    sq,
                    x,
                    y,
                    a + turn,
                    ((steps - k) * 0.4).floor(),
                    v * 0.8,
                );
            }
            k += 1.0;
        }
    }
    let mut i = 0.0;
    while i < n {
        let a = (i / n) * TAU + (rand.next() - 0.5) * 0.8;
        let steps = ((len / 1.3) * lerp(0.6, 1.0, rand.next()) * grow).floor();
        walk(c, &mut rand, sq, cx, cy, a, steps, gain);
        i += 1.0;
    }
}

fn spikes(c: &mut C, l: &Val) {
    let n = max(1.0, round(l.n("count", 5.0)));
    let mut rand = Rng::new(c.seed);
    let spread = l.n("spread", 0.75);
    let [h_a, h_b] = pair(l, "hgt", [0.55, 1.0]);
    let [w_a, w_b] = pair(l, "width", [0.1, 0.17]);
    let (grow, hold, gain, s) = (
        l.n("grow", 0.22),
        l.n("hold", 0.55),
        l.n("gain", 1.0),
        c.seed,
    );
    let mut i = 0.0;
    while i < n {
        let ii = i;
        i += 1.0;
        let off = if n == 1.0 {
            0.0
        } else {
            lerp(-1.0, 1.0, ii / (n - 1.0)) * spread + (rand.next() - 0.5) * 0.12
        };
        let hh = lerp(h_a, h_b, rand.next()) * (1.0 - (0.35 * off.abs()) / max(spread, 0.01));
        let w = lerp(w_a, w_b, rand.next());
        let lean = (rand.next() - 0.5) * 2.0 * l.n("lean", 12.0)
            + (off / max(spread, 0.01)) * l.n("fan", 14.0);
        let delay = (off.abs() / max(spread, 0.01)) * 0.12 + rand.next() * 0.05;
        let g = if c.looping {
            1.0
        } else {
            ease("outBack", (c.lt - delay) / grow)
        };
        if g <= 0.0 {
            continue;
        }
        let shat = if c.looping {
            0.0
        } else {
            smooth(hold, 1.0, c.lt)
        };
        let hc = hh * g * (1.0 - 0.3 * shat);
        c.field(
            1.0,
            hc + w + 0.05,
            l.n("x", 0.0) + off,
            l.n("y", 0.0),
            lean,
            |x, y| {
                let a = -y;
                if a < 0.0 || a > hc {
                    return 0.0;
                }
                let hw = w * (1.0 - a / hc);
                if x.abs() > hw {
                    return 0.0;
                }
                if shat > 0.0 && fbm(x * 8.0 + ii, y * 8.0, shat * 3.0, s, 2) < shat * 1.05 {
                    return 0.0;
                }
                let mut v = 0.45 + 0.5 * (a / hc);
                if x < 0.0 {
                    v -= 0.22;
                }
                if x > 0.0 && hw - x < 0.05 {
                    v += 0.25;
                }
                v * gain
            },
        );
    }
}

fn star(c: &mut C, l: &Val) {
    let k = c.lt;
    let pts = max(3.0, round(l.n("points", 8.0)));
    let (r, inner, sharp, jit) = (
        l.n("r", 0.75),
        l.n("inner", 0.22),
        l.n("sharp", 1.6),
        l.n("jitter", 0.45),
    );
    let (pop, s, gain) = (l.n("pop", 0.22), c.seed, l.n("gain", 1.1));
    let sc = ease("outBack", k / pop) * (1.0 - 0.25 * smooth(pop, 1.0, k));
    let hole = l.n("hollow", 0.9) * ease("outQuad", smooth(pop * 0.9, 1.0, k));
    if sc <= 0.01 {
        return;
    }
    let rot = l
        .opt("angle")
        .map_or_else(|| hash01(1.0, 2.0, 3.0, s) * 360.0, Val::num);
    c.field(
        l.n("squash", 1.0),
        r * 1.35,
        l.n("x", 0.0),
        l.n("y", 0.0),
        rot,
        |x, y| {
            let rad = hypot(x, y);
            let a = ((((m::atan2(y, x) / TAU) % 1.0) + 1.0) % 1.0) * pts;
            let i = round(a) % pts;
            let f = (a - round(a)).abs();
            let len = r * (1.0 - jit * hash01(i, 5.0, 0.0, s)) * sc;
            let rs = len * inner + (len - len * inner) * m::pow(max(0.0, 1.0 - f * 2.0), sharp);
            if rad > rs || rad < hole * len {
                return 0.0;
            }
            (1.0 - (rad / rs) * 0.6) * gain
        },
    );
}

fn streaks(c: &mut C, l: &Val) {
    let mode = match l.get("mode") {
        Some(v) if v.truthy() => v.js_string(),
        _ => "claw".into(),
    };
    let n = round(l.n("count", 3.0));
    let mut rand = Rng::new(c.seed);
    let gain = l.n("gain", 1.0);
    let (ax, ay) = (l.n("x", 0.0), l.n("y", 0.0));
    let mut i = 0.0;
    if mode == "claw" {
        let ang = l.n("angle", 55.0) * DEG;
        let (len, gap, curve, th) = (
            l.n("len", 1.4),
            l.n("gap", 0.22),
            l.n("curve", 0.18),
            l.n("th", 2.0),
        );
        let (dx, dy) = (m::cos(ang), m::sin(ang));
        let (nx, ny) = (-dy, dx);
        while i < n {
            let lt = c.lt - i * l.n("stagger", 0.04);
            let head = ease("outCubic", lt / 0.22);
            let tail = smooth(0.12, 0.8, lt);
            let ii = i;
            i += 1.0;
            if head <= tail {
                continue;
            }
            let off = (ii - (n - 1.0) / 2.0) * gap;
            let p0 = [
                ax - (dx * len) / 2.0 + nx * off,
                ay - (dy * len) / 2.0 + ny * off,
            ];
            let p2 = [
                ax + (dx * len) / 2.0 + nx * off,
                ay + (dy * len) / 2.0 + ny * off,
            ];
            let p1 = [ax + nx * (off + curve), ay + ny * (off + curve)];
            let steps = (len * c.s * 1.5).ceil();
            let mut mm = 0.0;
            while mm <= steps {
                let t = lerp(tail, head, mm / steps);
                let u = 1.0 - t;
                let bx = u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0];
                let by = u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1];
                let prof = m::sin(PI * (mm / steps));
                let r = max(0.5, th * prof * (1.0 - smooth(0.4, 1.0, c.lt) * 0.6));
                let (px, py) = (
                    round(c.w / 2.0 + bx * c.s - 0.5),
                    round(c.h / 2.0 + by * c.s - 0.5),
                );
                c.disc(px, py, r * 0.6, gain * (0.55 + 0.45 * prof));
                mm += 1.0;
            }
        }
    } else if mode == "speed" {
        let wu = c.w / c.s;
        let cyc = max(1.0, round(l.n("speed", 2.0)));
        let (dirx, diry) = (
            m::cos(l.n("angle", 180.0) * DEG),
            m::sin(l.n("angle", 180.0) * DEG),
        );
        while i < n {
            let yy = ay + (rand.next() - 0.5) * l.n("spanY", 0.8);
            let ln = lerp(0.2, 0.7, rand.next()) * l.n("len", 1.0);
            let off = rand.next();
            let t = if c.looping {
                (off + c.ph * cyc) % 1.0
            } else {
                (off * 0.3 + c.lt * cyc) % 1.0
            };
            let xx = lerp(-wu / 2.0 - ln, wu / 2.0 + ln, t) * -dirx;
            let v = gain * lerp(0.5, 1.0, rand.next());
            let (x0, y0, x1, y1) = (
                c.px(xx),
                c.py(yy),
                c.px(xx - dirx * ln),
                c.py(yy - diry * ln),
            );
            c.line(x0, y0, x1, y1, v, v * 0.3, 1.0);
            i += 1.0;
        }
    } else {
        let t = round(l.n("turns", 1.0));
        let turns = if truthy(t) { t } else { 1.0 };
        let dir = l.n("dir", 1.0);
        let sq = l.n("squash", 0.5);
        while i < n {
            let r = lerp(0.3, 1.0, rand.next()) * l.n("r", 0.8);
            let a0 = rand.next() * TAU;
            let span = lerp(0.6, 1.6, rand.next());
            let head = a0 + TAU * turns * if c.looping { c.ph } else { c.lt } * dir;
            let steps = (r * c.s * span).ceil();
            let mut mm = 0.0;
            while mm <= steps {
                let a = head - (mm / steps) * span * dir;
                let (px, py) = (c.px(ax + m::cos(a) * r), c.py(ay + m::sin(a) * r * sq));
                c.plot(px, py, gain * (1.0 - (mm / steps) * 0.7));
                mm += 1.0;
            }
            i += 1.0;
        }
    }
}

fn hex_edge(x: f64, y: f64, size: f64) -> f64 {
    let q = ((3f64.sqrt() / 3.0) * x - y / 3.0) / size;
    let r = ((2.0 / 3.0) * y) / size;
    let (cx, cz) = (q, r);
    let cy = -cx - cz;
    let (mut rx, mut ry, mut rz) = (round(cx), round(cy), round(cz));
    let (dx, dy, dz) = ((rx - cx).abs(), (ry - cy).abs(), (rz - cz).abs());
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }
    max(max((rx - cx).abs(), (ry - cy).abs()), (rz - cz).abs()) * 2.0
}

fn bubble(c: &mut C, l: &Val) {
    let (r, th, hex, fill, cell) = (
        l.n("r", 0.8),
        l.n("th", 0.07),
        l.n("hex", 0.4),
        l.n("fill", 0.16),
        l.n("cell", 0.17),
    );
    let sh = round(l.n("shimmer", 1.0));
    let sh = if truthy(sh) { sh } else { 1.0 };
    let gain = l.n("gain", 1.0);
    let mut env = 1.0;
    if !c.looping {
        env = ease("outBack", c.lt / 0.2) * (1.0 - smooth(0.8, 1.0, c.lt));
    }
    let (rr, hl) = (r * env, TAU * c.ph * sh - PI / 2.0);
    if rr < 0.02 {
        return;
    }
    c.field_l(l, rr + th + 0.05, |x, y| {
        let d = hypot(x, y);
        if d > rr + th {
            return 0.0;
        }
        if d > rr - th {
            return 0.95 * gain;
        }
        let mut da = m::atan2(y, x) - hl;
        da = m::atan2(m::sin(da), m::cos(da));
        if d > rr * 0.7 && da.abs() < 0.45 {
            return (0.45 + 0.5 * (1.0 - da.abs() / 0.45)) * gain;
        }
        if truthy(hex) && hex_edge(x, y, cell) > 0.84 {
            return hex * gain * (0.7 + 0.3 * (d / rr));
        }
        fill * gain * (0.6 + 0.6 * m::pow(d / rr, 2.0))
    });
}

fn flash(c: &mut C, l: &Val) {
    let dur = l.n("dur", 0.12);
    if c.lt > dur {
        return;
    }
    let k = c.lt / dur;
    let r = l.n("r", 0.6) * (0.7 + 0.3 * k);
    let gain = l.n("gain", 1.3);
    c.field_l(l, r + 0.05, |x, y| {
        let d = hypot(x, y) / r;
        if d > 1.0 {
            0.0
        } else {
            (1.0 - m::pow(d, 4.0)) * gain * (1.0 - 0.4 * k)
        }
    });
}

fn text(c: &mut C, l: &Val) {
    let s: Vec<char> = l
        .opt("text")
        .map_or_else(|| "CRIT!".to_string(), Val::js_string)
        .to_uppercase()
        .chars()
        .take(14)
        .collect();
    let k = c.lt;
    let base = max(1.0, round(l.n("scale", 1.0)));
    let pop = !matches!(l.get("pop"), Some(Val::Bool(false)));
    let sc = if k < 0.1 && pop { base + 1.0 } else { base };
    let rise = ease("outCubic", k) * l.n("rise", 0.35);
    let wpx = s.len() as f64 * 4.0 * sc - sc;
    let hpx = 5.0 * sc;
    let x0 = round(c.w / 2.0 + l.n("x", 0.0) * c.s - wpx / 2.0);
    let y0 = round(c.h / 2.0 + (l.n("y", 0.0) - rise) * c.s - hpx / 2.0);
    let (gain, fade_at) = (l.n("gain", 1.0), l.n("fadeStart", 0.7));
    let question = FONT.iter().find(|(ch, _)| *ch == '?').expect("?").1;
    for (ci, ch) in s.iter().enumerate() {
        let g = FONT
            .iter()
            .find(|(f, _)| f == ch)
            .map_or(question, |(_, g)| *g)
            .as_bytes();
        for r in 0..5 {
            for col in 0..3 {
                if g[r * 3 + col] != b'1' {
                    continue;
                }
                let mut yy = 0.0;
                while yy < sc {
                    let mut xx = 0.0;
                    while xx < sc {
                        let x = x0 + ci as f64 * 4.0 * sc + col as f64 * sc + xx;
                        let y = y0 + r as f64 * sc + yy;
                        xx += 1.0;
                        if k > fade_at && hash01(x, y, 1.0, c.seed) < smooth(fade_at, 1.0, k) {
                            continue;
                        }
                        c.plot(x, y, gain * (0.62 + 0.38 * (1.0 - r as f64 / 4.0)));
                    }
                    yy += 1.0;
                }
            }
        }
    }
}

fn tornado(c: &mut C, l: &Val) {
    let (w0, w1, hh, twist) = (
        l.n("w0", 0.12),
        l.n("w1", 0.6),
        l.n("hgt", 1.8),
        l.n("twist", 3.0),
    );
    let sp = round(l.n("spin", 2.0));
    let spin = if truthy(sp) { sp } else { 1.0 };
    let (freq, rough, sway) = (l.n("freq", 3.0), l.n("rough", 0.5), l.n("sway", 0.08));
    let (s, gain, p) = (c.seed, l.n("gain", 1.0), 8.0);
    let mut env = 1.0;
    if !c.looping {
        env = ease("outCubic", c.lt / 0.25) * (1.0 - smooth(0.75, 1.0, c.lt));
    }
    if env < 0.02 {
        return;
    }
    let ph = c.ph;
    c.field_l(l, hh + w1 + 0.2, |x, y| {
        let ly = -y / hh;
        if ly < 0.0 || ly > env {
            return 0.0;
        }
        let cx = sway * m::sin(TAU * (ly * 0.8 + ph));
        let hw = (w0 + (w1 - w0) * m::pow(ly, 1.3)) * env;
        let lx = (x - cx) / hw;
        if lx.abs() > 1.0 {
            return 0.0;
        }
        let wrap_a = m::asin(clamp(lx, -1.0, 1.0)) / PI;
        let stripe = 0.5 + 0.5 * m::cos(TAU * (ly * twist * 2.0 + wrap_a * 2.0 - ph * spin));
        let n = fbm_p(
            lx * 2.0,
            ly * hh * freq - ph * p * spin,
            0.3,
            s,
            2,
            0.0,
            p,
            0.0,
        );
        let body = 0.32 + 0.75 * stripe * (0.55 + 0.9 * (n - 0.5) * rough * 2.0);
        body * (1.0 - lx * lx * 0.55) * (1.0 - smooth(0.85, 1.0, ly / env)) * gain
    });
}

pub(super) fn draw(ty: &str, c: &mut C, l: &Val) {
    match ty {
        "burst" => burst(c, l),
        "ring" => ring(c, l),
        "flame" => flame(c, l),
        "beam" => beam(c, l),
        "slash" => slash(c, l),
        "orb" => orb(c, l),
        "comet" => comet(c, l),
        "vortex" => vortex(c, l),
        "pillar" => pillar(c, l),
        "rays" => rays(c, l),
        "glyph" => glyph(c, l),
        "puffs" => puffs(c, l),
        "particles" => particles(c, l),
        "bolt" => bolt(c, l),
        "cracks" => cracks(c, l),
        "spikes" => spikes(c, l),
        "star" => star(c, l),
        "streaks" => streaks(c, l),
        "bubble" => bubble(c, l),
        "flash" => flash(c, l),
        "text" => text(c, l),
        "tornado" => tornado(c, l),
        _ => {}
    }
}
