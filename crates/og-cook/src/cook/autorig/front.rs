//! The humanoid-front clips: idle, walk, run, jump, attack, hit, die and
//! the emotes wave, cheer and shrug, for a character facing the viewer.
//! Seen from the front a step is a knee lifting and the hips swaying over
//! the planted foot, so the feet move up and down under the body (their
//! IK targets, `ik-foot-l` and `ik-foot-r`) and the body sways, tilts and
//! counters from the hips up to the head.
//!
//! `-l` is the image's left, `-r` its right: an arm swung outwards turns
//! clockwise on the left and counter-clockwise on the right, so each side
//! scales its angles by [`out`].

use super::clips::{Clip, Curve, Measure, swing};
use super::side::{chains, free_side, holds};
use og_anim::{Clip2d, Rig2dAsset};

/// The two sides with their half cycle: the image's right steps first.
const SIDES: [(&str, f32); 2] = [("r", 0.0), ("l", 0.5)];

/// +1 on the image's right, -1 on its left: an outward turn's sign.
fn out(side: &str) -> f32 {
    if side == "r" { 1.0 } else { -1.0 }
}

/// An arm turned outwards by `upper`, its forearm by `lower` and its hand
/// by `hand` (degrees, outwards positive on either side). A holding hand
/// turns back by part of what the arm turns, so what it holds keeps its
/// line.
fn arm(
    c: &mut Clip,
    rig: &Rig2dAsset,
    side: &str,
    upper: impl Fn(f32) -> f32,
    lower: impl Fn(f32) -> f32,
    hand: impl Fn(f32) -> f32,
) {
    let o = out(side);
    let steady = holds(rig, side);
    c.rot(&format!("arm-upper-{side}"), |t| o * upper(t));
    c.rot(&format!("arm-lower-{side}"), |t| o * lower(t));
    c.rot(&format!("hand-{side}"), |t| {
        if steady {
            -0.6 * o * (upper(t) + lower(t))
        } else {
            o * hand(t)
        }
    });
}

/// The hips: `x` sways sideways and `y` bobs (rig pixels), `tilt` degrees.
fn hips(c: &mut Clip, x: impl Fn(f32) -> f32, y: impl Fn(f32) -> f32, tilt: impl Fn(f32) -> f32) {
    c.mov("pelvis", |t| [x(t), y(t)]);
    c.rot("pelvis", tilt);
}

fn idle(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "idle", 2.4, true);
    let l = m.leg;
    // A breath and a slow weight shift: the hips settle and drift a
    // little, the chest rises late, the head follows later still.
    hips(
        &mut c,
        |t| swing(t, 0.02 * l, 1.0, 0.0),
        |t| -0.012 * l * (0.5 - 0.5 * (std::f32::consts::TAU * 2.0 * t).cos()),
        |t| swing(t, 1.2, 1.0, 0.02),
    );
    c.shift("torso", |t| {
        [
            0.0,
            0.008 * l * (0.5 - 0.5 * (std::f32::consts::TAU * 2.0 * (t - 0.05)).cos()),
        ]
    });
    c.rot("torso", |t| swing(t, -1.6, 1.0, 0.08));
    c.rot("neck", |t| swing(t, 0.8, 1.0, 0.14));
    c.rot("head", |t| swing(t, 2.0, 1.0, 0.2));
    for (s, phase) in SIDES {
        arm(
            &mut c,
            rig,
            s,
            move |t| 4.0 + swing(t + phase * 0.2, 1.5, 2.0, 0.1),
            move |t| -4.0 + swing(t + phase * 0.2, 2.0, 2.0, 0.18),
            move |t| swing(t + phase * 0.2, 3.0, 2.0, 0.26),
        );
    }
    chains(&mut c, |t, k, s| {
        s * swing(t, -2.0 - k as f32, 1.0, 0.16 + 0.08 * k as f32)
    });
    c.done()
}

/// A step cycle seen from the front: the knee lifts `lift` leg lengths
/// over `swing_len` of the cycle, the hips sway `sway` leg lengths over
/// the planted foot and drop `drop` at each contact.
struct Gait {
    name: &'static str,
    duration: f32,
    lift: f32,
    swing_len: f32,
    sway: f32,
    drop: f32,
    /// Hips raised in the flight between steps (a run), leg lengths.
    flight: f32,
    /// How far each arm swings out and back, and its forearm's bend.
    arm: f32,
    bend: f32,
}

fn gait(rig: &Rig2dAsset, m: &Measure, g: &Gait) -> Clip2d {
    let mut c = Clip::new(rig, g.name, g.duration, true);
    let l = m.leg;
    let w = g.swing_len;
    // One foot over the cycle: planted from 0 to 1 - w, lifted through its
    // swing (highest at its middle), set down at 1; carried a little out
    // to the side as it passes.
    let fy = Curve::cycle(&[
        (0.0, 0.0),
        (1.0 - w, 0.0),
        (1.0 - w * 0.85, 0.35 * g.lift * l),
        (1.0 - w * 0.55, g.lift * l),
        (1.0 - w * 0.2, g.lift * 0.45 * l),
        (1.0, 0.0),
    ]);
    let tuck = Curve::cycle(&[
        (0.0, 0.0),
        (1.0 - w, 0.0),
        (1.0 - w * 0.55, 0.03 * l),
        (1.0, 0.0),
    ]);
    for (side, phase) in SIDES {
        let (fy, fy2, fy3, tuck) = (fy.clone(), fy.clone(), fy.clone(), tuck.clone());
        let o = out(side);
        let lift = (g.lift * l).max(1.0);
        c.mov(&format!("ik-foot-{side}"), move |t| {
            [o * tuck.at(t + phase), fy.at(t + phase)]
        });
        c.rot(&format!("foot-{side}"), move |t| {
            o * 4.0 * fy2.at(t + phase) / lift
        });
        // Seen from the front a knee lifts towards the viewer: near the
        // top of the lift the leg foreshortens, nearly straight, instead
        // of bending sideways across the other leg (lower down, where the
        // hips have swayed away from the foot, it needs its full length).
        let reach = m.leg.max(1.0);
        // Uniformly: a scale along the thigh alone would shear the turned
        // shin, which the leg's two-bone IK does not solve for.
        c.scale(&format!("leg-upper-{side}"), move |t| {
            let k = 1.0 - 0.8 * (fy3.at(t + phase) - 0.4 * lift).max(0.0) / reach;
            [k, k]
        });
    }
    // The hips over the planted foot: towards the right while the right
    // foot carries the body (the left lifted, around `over`), dropping just
    // after each contact, and tilting down on the lifted side; the chest
    // counters the tilt and the head counters the chest, late, so it stays
    // nearly level.
    let over = 0.5 - 0.5 * w;
    let sway = g.sway * l;
    // Highest as the lifted leg passes (a walk), or in the flight once
    // both feet are off the ground (a run: each step's stance ends at
    // `stance`).
    let stance = 2.0 * (1.0 - w);
    let d = g.drop * l;
    let bob = if stance < 1.0 {
        Curve::cycle(&[
            (0.0, -d),
            (0.3, -1.4 * d),
            (stance - 0.05, -1.1 * d),
            (0.5 + 0.5 * stance, g.flight * l - 0.3 * d),
            (1.0, -d),
        ])
    } else {
        Curve::cycle(&[(0.0, -d), (0.3, -1.4 * d), (0.65, -0.3 * d), (1.0, -d)])
    };
    let tilt = 4.0 * g.lift / 0.16;
    hips(
        &mut c,
        |t| swing(t, sway, 1.0, over + 0.03),
        |t| bob.at(2.0 * t),
        |t| swing(t, tilt, 1.0, over),
    );
    c.rot("torso", |t| swing(t, -0.8 * tilt, 1.0, over + 0.05));
    c.rot("neck", |t| swing(t, 0.2 * tilt, 1.0, over + 0.1));
    c.rot("head", |t| swing(t, 0.35 * tilt, 1.0, over + 0.14));
    // The arms swing against the legs: from the front a swing forwards
    // shows as the arm lifting out a little and the forearm bending in.
    for (side, phase) in SIDES {
        let k = if holds(rig, side) { 0.5 } else { 1.0 };
        let (a, b) = (g.arm, g.bend);
        arm(
            &mut c,
            rig,
            side,
            move |t| 6.0 + k * a * (0.5 + 0.5 * swing(t + phase, 1.0, 1.0, 0.0)),
            move |t| -b - k * 0.6 * b * (0.5 + 0.5 * swing(t + phase, 1.0, 1.0, 0.06)),
            move |t| swing(t + phase, 5.0, 1.0, 0.12),
        );
    }
    // A cape and hair swing against the sway, late; a tail too.
    let s = g.sway / 0.03;
    chains(&mut c, |t, k, sc| {
        sc * s * swing(t, -3.0 - 1.5 * k as f32, 1.0, over + 0.08 + 0.07 * k as f32)
    });
    c.event(0.0, "step");
    c.event(0.5, "step");
    c.done()
}

fn walk(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    gait(
        rig,
        m,
        &Gait {
            name: "walk",
            duration: 1.0,
            lift: 0.11,
            swing_len: 0.42,
            sway: 0.03,
            drop: 0.006,
            flight: 0.0,
            arm: 8.0,
            bend: 8.0,
        },
    )
}

fn run(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    gait(
        rig,
        m,
        &Gait {
            name: "run",
            duration: 0.64,
            lift: 0.17,
            swing_len: 0.6,
            sway: 0.01,
            drop: 0.035,
            flight: 0.02,
            arm: 16.0,
            bend: 55.0,
        },
    )
}

fn jump(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "jump", 1.1, false);
    let l = m.leg;
    // Anticipation, take-off, the tuck at the top, reaching down, landing.
    let k = [0.0, 0.18, 0.3, 0.46, 0.66, 0.8, 0.88, 1.0];
    let key = |v: [f32; 8]| Curve::keyed(&k, &v);
    let hips = key([0.0, -0.26, 0.04, 0.02, 0.02, 0.04, -0.22, 0.0].map(|v| v * l));
    // The toes leave the ground as the hips rise past standing.
    let feet = key([0.0, 0.0, 0.06, 0.3, 0.32, 0.06, 0.0, 0.0].map(|v| v * l));
    let spread = key([0.0, 0.03, -0.02, 0.05, 0.05, 0.02, 0.04, 0.0].map(|v| v * l));
    let arm = key([0.0, -12.0, 150.0, 120.0, 100.0, 60.0, 30.0, 0.0]);
    let fore = key([0.0, -10.0, 10.0, -20.0, -20.0, -10.0, -25.0, 0.0]);
    let head = key([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    c.mov("pelvis", |t| [0.0, hips.at(t)]);
    for (side, lead) in [("r", 0.0), ("l", 0.02)] {
        let (feet, feet2, spread) = (feet.clone(), feet.clone(), spread.clone());
        let o = out(side);
        c.mov(&format!("ik-foot-{side}"), move |t| {
            [o * spread.at(t - lead), feet.at(t - lead)]
        });
        c.rot(&format!("foot-{side}"), move |t| {
            -o * 12.0 * feet2.at(t - lead) / (0.3 * l)
        });
        let (arm, fore) = (arm.clone(), fore.clone());
        let kk = if holds(rig, side) { 0.6 } else { 1.0 };
        let upper = move |t: f32| kk * arm.at(t - lead);
        let lower = move |t: f32| kk * fore.at(t - lead - 0.03);
        self::arm(&mut c, rig, side, upper, lower, |_| 0.0);
    }
    c.rot("head", |t| head.at(t));
    let squash = key([0.0, -0.02, 0.03, 0.0, 0.0, 0.0, -0.02, 0.0]);
    c.shift("torso", |t| [0.0, squash.at(t - 0.02) * l]);
    let trail = key([0.0, 4.0, -18.0, -8.0, 6.0, 14.0, -10.0, 0.0]);
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.4 * kk as f32) * trail.at(t - 0.05 * (kk + 1) as f32)
    });
    c.event(0.3, "takeoff");
    c.event(0.88, "land");
    c.done()
}

fn attack(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "attack", 0.7, false);
    let l = m.leg;
    // The striking arm (the one holding the weapon) raised out and over
    // the head (0.3), chopped down across the body (0.42), following
    // through (0.54), recovering; the body twists into the blow.
    let side = if holds(rig, "l") && !holds(rig, "r") {
        "l"
    } else {
        "r"
    };
    let other = if side == "r" { "l" } else { "r" };
    let o = out(side);
    let k = [0.0, 0.3, 0.42, 0.54, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let arm = key([0.0, 160.0, -20.0, -36.0, 0.0]);
    let fore = key([0.0, 30.0, -10.0, -14.0, 0.0]);
    let off = key([0.0, 24.0, 40.0, 34.0, 0.0]);
    let torso = key([0.0, 6.0, -8.0, -9.0, 0.0]);
    let head = key([0.0, -3.0, 4.0, 4.0, 0.0]);
    let shift = key([0.0, 0.04, -0.05, -0.06, 0.0].map(|v| v * l));
    let drop = key([0.0, 0.0, -0.08, -0.09, 0.0].map(|v| v * l));
    self::arm(
        &mut c,
        rig,
        side,
        |t| arm.at(t),
        |t| fore.at(t - 0.02),
        |_| 0.0,
    );
    self::arm(
        &mut c,
        rig,
        other,
        |t| off.at(t - 0.03),
        |t| -0.5 * off.at(t - 0.06),
        |_| 0.0,
    );
    c.rot("torso", |t| o * torso.at(t));
    c.rot("head", |t| o * head.at(t - 0.04));
    hips(
        &mut c,
        |t| o * shift.at(t),
        |t| drop.at(t),
        |t| o * 0.5 * torso.at(t - 0.02),
    );
    let trail = key([0.0, -8.0, 14.0, 10.0, 0.0]);
    chains(&mut c, |t, kk, sc| {
        o * sc * (1.0 + 0.5 * kk as f32) * trail.at(t - 0.05 * (kk + 1) as f32)
    });
    c.in_front(&[
        format!("arm-upper-{side}"),
        format!("arm-lower-{side}"),
        format!("hand-{side}"),
        "weapon".into(),
        "shield".into(),
    ]);
    c.event(0.42, "hit");
    c.done()
}

fn hit(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "hit", 0.45, false);
    let l = m.leg;
    // Struck from the image's right: knocked left, the head snapping
    // over, the arms flung out, a flash of red that fades.
    let k = [0.0, 0.18, 0.32, 0.6, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let shove = key([0.0, -0.08, -0.06, -0.02, 0.0].map(|v| v * l));
    let dip = key([0.0, -0.05, -0.04, -0.01, 0.0].map(|v| v * l));
    let tilt = key([0.0, 8.0, 6.0, 2.0, 0.0]);
    let head = key([0.0, 14.0, 10.0, 3.0, 0.0]);
    let arms = key([0.0, 30.0, 20.0, 5.0, 0.0]);
    hips(
        &mut c,
        |t| shove.at(t),
        |t| dip.at(t),
        |t| -0.4 * tilt.at(t),
    );
    c.rot("torso", |t| tilt.at(t - 0.02));
    c.rot("head", |t| head.at(t - 0.06));
    for (side, lag) in [("r", 0.03), ("l", 0.05)] {
        let (a1, a2) = (arms.clone(), arms.clone());
        arm(
            &mut c,
            rig,
            side,
            move |t| a1.at(t - lag),
            move |t| -0.6 * a2.at(t - lag - 0.04),
            |_| 0.0,
        );
    }
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.4 * kk as f32) * tilt.at(t - 0.06 * (kk + 1) as f32)
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
    let l = m.leg;
    // Struck (0.12), the knees buckle (0.4), toppling over to the image's
    // left (0.72), hitting the ground (0.8), a small bounce, still.
    let k = [0.0, 0.12, 0.4, 0.72, 0.8, 0.88, 1.0];
    let key = |v: [f32; 7]| Curve::keyed(&k, &v);
    let fall = key([0.0, -3.0, 6.0, 84.0, 90.0, 87.0, 90.0]);
    // Lying on its side: lifted by how far its left side reaches.
    let lift = key([0.0, 0.0, 0.0, 0.85, 1.0, 1.04, 1.0].map(|v| v * m.back));
    let hips = key([0.0, -0.02, -0.3, -0.12, -0.02, -0.03, -0.02].map(|v| v * l));
    let torso = key([0.0, -10.0, 8.0, -4.0, 0.0, 2.0, 0.0]);
    let head = key([0.0, -16.0, 12.0, -10.0, 4.0, -4.0, -2.0]);
    let arm = key([0.0, 40.0, 16.0, 30.0, 10.0, 14.0, 12.0]);
    let fore = key([0.0, -10.0, -25.0, -30.0, -8.0, -14.0, -12.0]);
    c.rot("root", |t| fall.at(t));
    c.mov("root", |t| [0.0, lift.at(t)]);
    for side in ["l", "r"] {
        let lift = lift.clone();
        let ankle = m.ankle;
        c.mov(&format!("ik-foot-{side}"), move |t| {
            let f = lift.at(t) / m.back.max(1.0);
            [-(lift.at(t) - ankle * f), 0.0]
        });
    }
    c.mov("pelvis", |t| [0.0, hips.at(t)]);
    c.rot("torso", |t| torso.at(t));
    c.rot("head", |t| head.at(t - 0.04));
    for (side, lag) in [("r", 0.03), ("l", 0.05)] {
        let (arm, fore) = (arm.clone(), fore.clone());
        self::arm(
            &mut c,
            rig,
            side,
            move |t| arm.at(t - lag),
            move |t| fore.at(t - lag - 0.03),
            |_| 0.0,
        );
    }
    let flop = key([0.0, 6.0, 4.0, -24.0, 16.0, 5.0, 8.0]);
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.5 * kk as f32) * flop.at(t - 0.04 * (kk + 1) as f32)
    });
    c.event(0.8, "fall");
    c.done()
}

fn wave(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "wave", 1.5, true);
    let l = m.leg;
    // The free arm raised out to the side and up, the forearm waving
    // three times over the head's side; the hips shift onto the other
    // foot and the head tilts towards the wave.
    let free = free_side(rig);
    let other = if free == "r" { "l" } else { "r" };
    let o = out(free);
    arm(
        &mut c,
        rig,
        free,
        |t| 125.0 + swing(t, 5.0, 3.0, 0.05),
        |t| 30.0 + swing(t, 26.0, 3.0, 0.0),
        |t| swing(t, 14.0, 3.0, 0.04),
    );
    arm(
        &mut c,
        rig,
        other,
        |t| 5.0 + swing(t, 2.0, 1.0, 0.2),
        |t| -6.0 + swing(t, 3.0, 1.0, 0.28),
        |_| 0.0,
    );
    hips(
        &mut c,
        |t| -o * (0.03 * l + swing(t, 0.008 * l, 1.0, 0.0)),
        |t| -0.015 * l + swing(t, 0.008 * l, 2.0, 0.0),
        |t| o * (3.0 + swing(t, 1.0, 1.0, 0.04)),
    );
    c.rot("torso", |t| -o * (4.0 + swing(t, 1.0, 3.0, 0.08)));
    c.rot("head", |t| o * (6.0 + swing(t, 3.0, 3.0, 0.15)));
    chains(&mut c, |t, k, sc| {
        sc * o * (swing(t, 3.0 + 2.0 * k as f32, 3.0, 0.1 + 0.07 * k as f32) - 2.0)
    });
    c.in_front(&[
        format!("arm-upper-{free}"),
        format!("arm-lower-{free}"),
        format!("hand-{free}"),
    ]);
    c.done()
}

fn cheer(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "cheer", 1.0, true);
    let l = m.leg;
    // Both arms up in a V, pumping twice, bouncing on the knees.
    hips(
        &mut c,
        |_| 0.0,
        |t| -0.06 * l * (0.5 - 0.5 * (std::f32::consts::TAU * 2.0 * t).cos()),
        |t| swing(t, 2.0, 1.0, 0.0),
    );
    for (side, lag) in [("r", 0.0), ("l", 0.03)] {
        arm(
            &mut c,
            rig,
            side,
            move |t| 140.0 + swing(t, 10.0, 2.0, lag),
            move |t| 10.0 + swing(t, 18.0, 2.0, lag + 0.06),
            move |t| swing(t, 10.0, 2.0, lag + 0.1),
        );
    }
    c.rot("torso", |t| swing(t, 2.0, 1.0, 0.06));
    c.rot("head", |t| swing(t, 5.0, 1.0, 0.12));
    chains(&mut c, |t, k, sc| {
        sc * swing(t, 2.0 + 1.5 * k as f32, 2.0, 0.08 * (k + 1) as f32)
    });
    c.done()
}

fn shrug(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "shrug", 1.2, true);
    let l = m.leg;
    // The arms lift out with the forearms turned up and open, the
    // shoulders (the chest) rising, the head tipping to one side; held,
    // then let fall.
    let k = [0.0, 0.25, 0.6, 0.85, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let up = key([0.0, 1.0, 1.0, 0.0, 0.0]);
    for (side, lag) in [("r", 0.0), ("l", 0.03)] {
        let up = up.clone();
        let up2 = up.clone();
        arm(
            &mut c,
            rig,
            side,
            move |t| 6.0 + 28.0 * up.at(t - lag),
            move |t| 75.0 * up2.at(t - lag - 0.03),
            |_| 0.0,
        );
    }
    c.shift("torso", |t| [0.0, 0.03 * l * up.at(t - 0.02)]);
    c.rot("head", |t| 12.0 * up.at(t - 0.06));
    c.mov("pelvis", |t| [0.0, -0.01 * l * up.at(t)]);
    chains(&mut c, |t, k, sc| {
        sc * -4.0 * (1.0 + k as f32) * up.at(t - 0.06 * (k + 1) as f32)
    });
    c.done()
}

/// Every humanoid-front clip, in the library's order.
pub fn clips(rig: &Rig2dAsset, m: &Measure) -> Vec<Clip2d> {
    vec![
        idle(rig, m),
        walk(rig, m),
        run(rig, m),
        jump(rig, m),
        attack(rig, m),
        hit(rig, m),
        die(rig, m),
        wave(rig, m),
        cheer(rig, m),
        shrug(rig, m),
    ]
}
