//! The creature clips: idle, walk, run, jump, attack, hit, die and the
//! emotes happy, wiggle and look, for a simple creature (a body, perhaps
//! a head and eyes, and limbs, a tail or wings that swing as chains).
//! Its life is squash and stretch: the body is scaled (its bone points
//! up, so x is its height and y its width) about its middle and moved so
//! its bottom stays on the ground, while the rigid parts on it keep their
//! shape (the rig's `rigid-*` constraints) and the limbs ripple through
//! their chains, each bone later than the one before.

use super::clips::{Clip, Curve, Measure, origin, slot_box, swing};
use og_anim::{Clip2d, Rig2dAsset};
use std::f32::consts::TAU;

/// The body squashed to `s` of its height (and widened to keep its
/// volume), plus `lift` and `shift` (rig pixels) and a lean of `lean`
/// degrees (positive leans back, away from the right), its bottom kept on
/// the ground: lowered by what the squash takes off its lower half and
/// raised by what the lean dips below it (the body taken as an ellipse
/// through its box).
struct Body {
    /// From the body's origin down to its bottom, and its half width.
    below: f32,
    half: f32,
}

impl Body {
    fn new(rig: &Rig2dAsset) -> Body {
        let o = origin(rig, "body").unwrap_or([0.0, 0.0]);
        let b = slot_box(rig, "body").unwrap_or([o[0], o[1], o[0], o[1]]);
        Body {
            below: (o[1] - b[1]).max(0.0),
            half: 0.5 * (b[2] - b[0]),
        }
    }

    fn drive(
        &self,
        c: &mut Clip,
        squash: impl Fn(f32) -> f32,
        lift: impl Fn(f32) -> f32,
        shift: impl Fn(f32) -> f32,
        lean: impl Fn(f32) -> f32,
    ) {
        let (below, half) = (self.below, self.half);
        let across = |s: f32| 1.0 / s.sqrt();
        c.scale("body", |t| {
            let s = squash(t);
            [s, across(s)]
        });
        c.mov("body", |t| {
            let (s, r) = (squash(t), lean(t).to_radians());
            let low = ((s * below * r.cos()).powi(2) + (across(s) * half * r.sin()).powi(2)).sqrt();
            [shift(t), lift(t) + low - below]
        });
        c.rot("body", lean);
    }
}

/// Every limb (and the tail and wings) rippling: bone `k` of limb `n`
/// turned by `f(fraction, n, k)`.
fn limbs(c: &mut Clip, f: impl Fn(f32, usize, usize) -> f32) {
    for n in 1..=8 {
        c.chain(&format!("limb-{n}"), |t, k| f(t, n, k));
    }
    c.chain("tail", |t, k| 1.3 * f(t, 0, k));
    c.chain("wing-l", |t, k| f(t, 9, k));
    c.chain("wing-r", |t, k| -f(t, 9, k));
}

/// Alternate limbs swing in opposite phase.
fn alt(n: usize) -> f32 {
    if n.is_multiple_of(2) { 0.5 } else { 0.0 }
}

fn idle(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "idle", 2.0, true);
    let h = m.height;
    // Breathing: a slow squash and stretch, a gentle sway, the limbs
    // drifting like weeds in a current.
    Body::new(rig).drive(
        &mut c,
        |t| 1.0 + 0.035 * (TAU * t).sin(),
        |_| 0.0,
        |_| 0.0,
        |t| swing(t, 1.5, 1.0, 0.2),
    );
    c.rot("head", |t| swing(t, 2.0, 1.0, 0.3));
    limbs(&mut c, |t, n, k| {
        swing(
            t,
            4.0 + 3.0 * k as f32,
            1.0,
            0.1 * n as f32 + 0.12 * k as f32,
        )
    });
    let _ = h;
    c.done()
}

/// A hop cycle: `hops` a cycle, rising `rise` of the height between
/// landings, squashed to `squash` on landing and stretched to `stretch`
/// in the air, leaning `lean` degrees into the travel.
struct Hop {
    name: &'static str,
    duration: f32,
    hops: f32,
    rise: f32,
    squash: f32,
    stretch: f32,
    lean: f32,
    swing: f32,
}

fn hop(rig: &Rig2dAsset, m: &Measure, g: &Hop) -> Clip2d {
    let mut c = Clip::new(rig, g.name, g.duration, true);
    let h = m.height;
    // Per hop: landed and squashing (0 to 0.18), pushing off stretched
    // (0.3), the top of the arc (0.62), stretched coming down (0.9).
    let lift = Curve::cycle(&[
        (0.0, 0.0),
        (0.18, 0.0),
        (0.3, 0.25 * g.rise * h),
        (0.62, g.rise * h),
        (0.9, 0.3 * g.rise * h),
        (1.0, 0.0),
    ]);
    let squash = Curve::cycle(&[
        (0.0, 1.0),
        (0.1, g.squash),
        (0.24, 1.0),
        (0.34, g.stretch),
        (0.62, 1.0),
        (0.9, 0.5 + 0.5 * g.stretch),
        (1.0, 1.0),
    ]);
    let hops = g.hops;
    let (lean, sw) = (g.lean, g.swing);
    Body::new(rig).drive(
        &mut c,
        |t| squash.at(hops * t),
        |t| lift.at(hops * t),
        |_| 0.0,
        |t| -lean + swing(t, 0.4 * lean.abs().max(3.0), hops, 0.1),
    );
    c.rot("head", |t| 0.6 * lean + swing(t, 3.0, hops, 0.2));
    // The limbs swing in alternation, once a cycle, and stream back in
    // the air.
    limbs(&mut c, |t, n, k| {
        let air = lift.at(hops * t - 0.05 * k as f32) / (g.rise * h).max(1.0);
        sw * (swing(t, 1.0 + 0.4 * k as f32, 1.0, alt(n) + 0.06 * k as f32)
            + 0.6 * air * (1.0 + k as f32))
    });
    c.event(0.0, "step");
    if hops > 1.5 {
        c.event(0.5, "step");
    }
    c.done()
}

fn walk(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    hop(
        rig,
        m,
        &Hop {
            name: "walk",
            duration: 1.0,
            hops: 2.0,
            rise: 0.05,
            squash: 0.9,
            stretch: 1.05,
            lean: 4.0,
            swing: 12.0,
        },
    )
}

fn run(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    hop(
        rig,
        m,
        &Hop {
            name: "run",
            duration: 0.56,
            hops: 1.0,
            rise: 0.28,
            squash: 0.78,
            stretch: 1.16,
            lean: 12.0,
            swing: 16.0,
        },
    )
}

fn jump(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "jump", 1.2, false);
    let h = m.height;
    // A deep squash (0.2), launching stretched (0.3), round at the top
    // (0.5), stretched falling (0.72), landing squashed flat (0.8), an
    // overshoot back up (0.9), still.
    let k = [0.0, 0.2, 0.3, 0.5, 0.72, 0.8, 0.9, 1.0];
    let key = |v: [f32; 8]| Curve::keyed(&k, &v);
    let squash = key([1.0, 0.68, 1.3, 1.0, 1.2, 0.7, 1.08, 1.0]);
    let lift = key([0.0, 0.0, 0.25, 0.6, 0.25, 0.0, 0.0, 0.0].map(|v| v * h));
    let lean = key([0.0, 4.0, -4.0, 0.0, 2.0, 0.0, 0.0, 0.0]);
    Body::new(rig).drive(
        &mut c,
        |t| squash.at(t),
        |t| lift.at(t),
        |_| 0.0,
        |t| lean.at(t),
    );
    let flare = key([0.0, -8.0, 20.0, 6.0, -18.0, 12.0, -4.0, 0.0]);
    limbs(&mut c, |t, _, kk| {
        (1.0 + 0.5 * kk as f32) * flare.at(t - 0.04 * (kk + 1) as f32)
    });
    c.event(0.3, "takeoff");
    c.event(0.8, "land");
    c.done()
}

fn attack(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "attack", 0.7, false);
    let h = m.height;
    // Rearing back, tall (0.3); lunging forwards, low and wide, the limbs
    // whipping forwards (0.45); held at full reach; snapping back.
    let k = [0.0, 0.3, 0.45, 0.58, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let squash = key([1.0, 1.18, 0.8, 0.86, 1.0]);
    let shift = key([0.0, -0.12, 0.3, 0.28, 0.0].map(|v| v * h));
    let lean = key([0.0, 14.0, -18.0, -14.0, 0.0]);
    Body::new(rig).drive(
        &mut c,
        |t| squash.at(t),
        |_| 0.0,
        |t| shift.at(t),
        |t| lean.at(t),
    );
    c.rot("head", |t| -0.4 * lean.at(t - 0.04));
    let whip = key([0.0, 22.0, -30.0, -20.0, 0.0]);
    limbs(&mut c, |t, _, kk| {
        (1.0 + 0.6 * kk as f32) * whip.at(t - 0.04 * (kk + 1) as f32)
    });
    c.event(0.45, "hit");
    c.done()
}

fn hit(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "hit", 0.45, false);
    let h = m.height;
    // Squashed by the blow and knocked back, wobbling, a flash of red.
    let k = [0.0, 0.12, 0.3, 0.55, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let squash = key([1.0, 0.8, 1.1, 0.96, 1.0]);
    let back = key([0.0, -0.12, -0.09, -0.02, 0.0].map(|v| v * h));
    let lean = key([0.0, 14.0, 6.0, -2.0, 0.0]);
    Body::new(rig).drive(
        &mut c,
        |t| squash.at(t),
        |_| 0.0,
        |t| back.at(t),
        |t| lean.at(t),
    );
    limbs(&mut c, |t, _, kk| {
        (1.0 + 0.5 * kk as f32) * lean.at(t - 0.05 * (kk + 1) as f32)
    });
    let fade = key([0.4, 0.2, 0.05, 0.0, 0.0]);
    c.flash(|t| {
        let f = fade.at(t);
        [f, f * 0.25, f * 0.25, 0.0]
    });
    c.done()
}

fn die(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "die", 1.4, false);
    let h = m.height;
    // A shudder (to 0.3), then melting: slumping lower and wider (0.75)
    // until it lies in a puddle, the limbs sagging, its colour fading.
    let k = [0.0, 0.1, 0.2, 0.3, 0.75, 0.85, 1.0];
    let key = |v: [f32; 7]| Curve::keyed(&k, &v);
    let squash = key([1.0, 0.9, 1.06, 0.95, 0.42, 0.46, 0.44]);
    let lean = key([0.0, 6.0, -6.0, 3.0, 0.0, 0.0, 0.0]);
    let sag = key([0.0, 4.0, -4.0, 0.0, 24.0, 20.0, 22.0]);
    let _ = h;
    Body::new(rig).drive(&mut c, |t| squash.at(t), |_| 0.0, |_| 0.0, |t| lean.at(t));
    limbs(&mut c, |t, n, kk| {
        let side = if n.is_multiple_of(2) { -1.0 } else { 1.0 };
        side * (1.0 + 0.4 * kk as f32) * sag.at(t - 0.04 * (kk + 1) as f32)
    });
    let grey = key([0.0, 0.0, 0.0, 0.0, 0.3, 0.32, 0.32]);
    c.flash(|t| {
        let g = grey.at(t);
        [-g, -g, -0.6 * g, 0.0]
    });
    c.event(0.75, "fall");
    c.done()
}

fn happy(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "happy", 1.0, true);
    let h = m.height;
    // Two quick bounces on the spot, stretching up at each, the limbs
    // waving fast overhead.
    let lift = Curve::cycle(&[
        (0.0, 0.0),
        (0.15, 0.0),
        (0.45, 0.16 * h),
        (0.85, 0.0),
        (1.0, 0.0),
    ]);
    let squash = Curve::cycle(&[
        (0.0, 1.0),
        (0.08, 0.84),
        (0.22, 1.14),
        (0.45, 1.0),
        (0.85, 1.04),
        (1.0, 1.0),
    ]);
    Body::new(rig).drive(
        &mut c,
        |t| squash.at(2.0 * t),
        |t| lift.at(2.0 * t),
        |_| 0.0,
        |t| swing(t, 5.0, 1.0, 0.0),
    );
    c.rot("head", |t| swing(t, 6.0, 1.0, 0.1));
    limbs(&mut c, |t, n, k| {
        swing(t, 14.0 + 6.0 * k as f32, 4.0, alt(n) + 0.05 * k as f32)
    });
    c.done()
}

fn wiggle(rig: &Rig2dAsset, _: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "wiggle", 1.2, true);
    // Side to side, stretching as it leans, the limbs swaying against it.
    Body::new(rig).drive(
        &mut c,
        |t| 1.0 + 0.06 * (TAU * 2.0 * t).cos(),
        |_| 0.0,
        |_| 0.0,
        |t| swing(t, 12.0, 1.0, 0.0),
    );
    c.rot("head", |t| swing(t, -6.0, 1.0, 0.06));
    limbs(&mut c, |t, _, k| {
        swing(t, -(10.0 + 6.0 * k as f32), 1.0, 0.06 * (k + 1) as f32)
    });
    c.done()
}

fn look(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "look", 2.0, true);
    let h = m.height;
    // The eyes (and the head) dart to one side, hold, dart to the other,
    // the body leaning after them.
    let glance = Curve::cycle(&[
        (0.0, 0.0),
        (0.08, 1.0),
        (0.4, 1.0),
        (0.48, -1.0),
        (0.8, -1.0),
        (0.88, 0.0),
        (1.0, 0.0),
    ]);
    let d = 0.035 * h;
    for eye in ["eye-l", "eye-r"] {
        let g = glance.clone();
        c.shift(eye, move |t| [g.at(t) * d, 0.0]);
    }
    c.rot("head", |t| -4.0 * glance.at(t - 0.04));
    Body::new(rig).drive(
        &mut c,
        |t| 1.0 + 0.02 * (TAU * t).sin(),
        |_| 0.0,
        |_| 0.0,
        |t| -3.0 * glance.at(t - 0.08),
    );
    limbs(&mut c, |t, n, k| {
        swing(
            t,
            3.0 + 2.0 * k as f32,
            1.0,
            0.1 * n as f32 + 0.1 * k as f32,
        )
    });
    c.done()
}

/// Every creature clip, in the library's order.
pub fn clips(rig: &Rig2dAsset, m: &Measure) -> Vec<Clip2d> {
    vec![
        idle(rig, m),
        walk(rig, m),
        run(rig, m),
        jump(rig, m),
        attack(rig, m),
        hit(rig, m),
        die(rig, m),
        happy(rig, m),
        wiggle(rig, m),
        look(rig, m),
    ]
}
