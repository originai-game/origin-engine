//! Built-in keyframe animations (`animate-*`) and cubic-bezier sampling.

use super::*;

/// A built-in keyframe animation (`animate-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyframes {
    /// Tailwind's: one turn clockwise.
    Spin,
    /// Tailwind's: scales to 2× and fades out.
    Ping,
    /// Tailwind's: fades to half opacity and back.
    Pulse,
    /// Tailwind's: jumps up a quarter of its height and lands.
    Bounce,
    /// Ours: in from opacity 0 at 0.9× on a spring.
    Pop,
    /// Ours: in from opacity 0 at 1.4×.
    Stamp,
    /// Ours: from 1.15× back to its size.
    Beat,
    /// Ours: settles from 1.4×, then rises 48 px and fades out, and stays
    /// out (a floating number).
    Float,
    /// Ours: sweeps across from -100% to 420% of its width.
    Shimmer,
    /// Ours: in from opacity 0, rising from 12 px below (a list's row).
    Rise,
    /// Ours: gathers to 1.08× over its first 40% (eased in, the charge),
    /// jumps to 1.15× and settles back to its size (the burst's pop).
    Charge,
    /// Ours: swells to 1.08× and back once (a control that became ready).
    Nudge,
    /// Ours: swells to 1.05× and back, forever (a call to action waiting
    /// on a tap: a claimable reward, a recommended button).
    Breathe,
    /// Ours: damped horizontal shake (hit reaction, denied input, low-ammo).
    Shake,
    /// Ours: brief opacity flash that decays to 0 and stays out (damage / hit flash).
    Flash,
    /// Ours: brief kickback and springy recovery (shot / strike feedback).
    Recoil,
}

/// `animate-*` and how it is timed (the class resolver's presets, or
/// `animation-*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loop {
    pub keyframes: Keyframes,
    /// Seconds per iteration.
    pub duration: f32,
    /// Seconds before it starts; it holds its first pose meanwhile.
    pub delay: f32,
    /// Cubic Bézier over each keyframe segment; `Bounce` keeps its own.
    pub ease: [f32; 4],
    /// `None` repeats forever.
    pub iterations: Option<u32>,
    /// Theme-driven peak amplitude (`scale-*` for Breathe/Float/Beat,
    /// `opacity-flash` for Flash); 0 uses the keyframe's default.
    pub peak: f32,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub(super) fn bezier(e: [f32; 4], x: f32) -> f32 {
    Easing::cubic(e[0], e[1], e[2], e[3]).eval(x.clamp(0.0, 1.0))
}

/// The keyframes' pose `elapsed` seconds after they started, over `s`:
/// whether they still play, and whether they hold a pose at all. Reduced
/// motion keeps the opacity and drops the movement.
pub(super) fn pose(
    lp: &Loop,
    elapsed: f64,
    s: &mut Skin,
    size: [f32; 2],
    reduced: bool,
) -> (bool, bool) {
    use Keyframes as K;
    let e = (elapsed - lp.delay as f64).max(0.0);
    let n = e / lp.duration.max(1e-3) as f64;
    let (p, playing) = match lp.iterations {
        Some(k) if n >= k as f64 => (1.0, false),
        _ => (n.fract() as f32, true),
    };
    // Pops, stamps, beats, shakes and recoils end where they rest;
    // floats and flashes stay faded out.
    if !playing && !matches!(lp.keyframes, K::Float | K::Flash) {
        return (false, false);
    }
    let ez = |x: f32| bezier(lp.ease, x);
    let mut scale = 1.0;
    let mut shift = [0.0f32; 2];
    let mut turn = 0.0;
    let o = &mut s.opacity;
    match lp.keyframes {
        K::Spin => turn = std::f32::consts::TAU * ez(p),
        K::Ping => {
            let q = ez((p / 0.75).min(1.0));
            scale = 1.0 + q;
            *o *= 1.0 - q;
        }
        K::Pulse => {
            let q = ez(if p < 0.5 { p * 2.0 } else { 2.0 - p * 2.0 });
            *o = lerp(*o, 0.5, q);
        }
        K::Bounce => {
            let q = if p < 0.5 {
                1.0 - bezier([0.8, 0.0, 1.0, 1.0], p * 2.0)
            } else {
                bezier([0.0, 0.0, 0.2, 1.0], p * 2.0 - 1.0)
            };
            shift[1] = -0.25 * size[1] * q;
        }
        K::Pop => {
            let q = ez(p);
            *o *= q.clamp(0.0, 1.0);
            scale = lerp(0.9, 1.0, q);
        }
        K::Stamp => {
            let q = ez(p);
            *o *= q;
            scale = lerp(1.4, 1.0, q);
        }
        K::Beat => {
            let peak = if lp.peak > 0.0 { lp.peak } else { 1.15 };
            scale = lerp(peak, 1.0, ez(p));
        }
        K::Float => {
            if p < 0.2 {
                let peak = if lp.peak > 0.0 { lp.peak } else { 1.4 };
                scale = lerp(peak, 1.0, ez(p / 0.2));
            } else {
                let q = ez((p - 0.2) / 0.8);
                shift[1] = -48.0 * q;
                *o *= 1.0 - q;
            }
        }
        K::Shimmer => shift[0] = size[0] * lerp(-1.0, 4.2, ez(p)),
        K::Nudge => scale = 1.0 + 0.08 * og_core::math::sin(std::f32::consts::PI * ez(p)),
        K::Breathe => {
            let peak = if lp.peak > 0.0 { lp.peak } else { 1.05 };
            scale = 1.0 + (peak - 1.0) * og_core::math::sin(std::f32::consts::PI * ez(p));
        }
        K::Rise => {
            let q = ez(p);
            *o *= q.clamp(0.0, 1.0);
            shift[1] = lerp(12.0, 0.0, q);
        }
        K::Charge => {
            scale = if p < 0.4 {
                let q = p / 0.4;
                lerp(1.0, 1.08, q * q)
            } else {
                lerp(1.15, 1.0, ez((p - 0.4) / 0.6))
            }
        }
        K::Shake => {
            let damp = (1.0 - p).max(0.0);
            shift[0] = 6.0 * damp * og_core::math::sin(p * 3.0 * std::f32::consts::TAU);
        }
        K::Flash => {
            let peak = if lp.peak > 0.0 { lp.peak } else { 0.6 };
            *o *= lerp(peak, 0.0, ez(p));
        }
        K::Recoil => {
            if p < 0.25 {
                let q = ez(p / 0.25);
                scale = lerp(1.0, 0.92, q);
                shift[1] = lerp(0.0, 3.0, q);
            } else {
                let q = ez((p - 0.25) / 0.75);
                scale = lerp(0.92, 1.0, q);
                shift[1] = lerp(3.0, 0.0, q);
            }
        }
    }
    if !reduced {
        let t = &mut s.transform;
        t.scale = [t.scale[0] * scale, t.scale[1] * scale];
        t.translate = [t.translate[0] + shift[0], t.translate[1] + shift[1]];
        t.rotate += turn;
    }
    (playing, true)
}
