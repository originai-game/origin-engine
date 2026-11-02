//! The quadruped clips: idle, walk, run, jump, attack, hit, die and the
//! emotes howl, play and sniff, for a four-legged animal facing right.
//! Each paw is placed by its leg's IK target (`ik-paw-front-l`, ...): a
//! walk sets them down one after another (hind left, fore left, hind
//! right, fore right), a run bounds with the hind pair and then the fore
//! pair, and the body pitches and bobs over them while the head counters
//! the body to stay level and the tail, ears and mane follow late.
//!
//! Angles are degrees counter-clockwise added to the setup pose: a
//! positive turn swings a leg forwards, lifts the head and raises the
//! body's front.

use super::clips::{Clip, Curve, Measure, origin, slot_box, swing};
use og_anim::{Clip2d, Rig2dAsset};

/// The legs in a walk's footfall order, with each one's phase.
const WALK: [(&str, &str, f32); 4] = [
    ("back", "l", 0.0),
    ("front", "l", 0.25),
    ("back", "r", 0.5),
    ("front", "r", 0.75),
];

fn ik(end: &str, side: &str) -> String {
    format!("ik-paw-{end}-{side}")
}

fn paw(end: &str, side: &str) -> String {
    format!("paw-{end}-{side}")
}

/// The tail through its chain: `f(fraction, bone)`, and the ears and
/// mane by `ears(fraction)`.
fn trailing(c: &mut Clip, tail: impl Fn(f32, usize) -> f32, ears: impl Fn(f32) -> f32) {
    c.chain("tail", tail);
    c.chain("mane", |t, k| 0.5 * ears(t - 0.04 * k as f32));
    c.rot("ear-l", |t| ears(t - 0.03));
    c.rot("ear-r", &ears);
}

fn idle(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "idle", 2.4, true);
    let l = m.leg;
    // A breath through the body, the head looking about slowly, the tail
    // swaying and an ear flicking once.
    c.mov("body", |t| {
        [
            0.0,
            -0.04 * l + 0.012 * l * (0.5 - 0.5 * (std::f32::consts::TAU * 2.0 * t).cos()),
        ]
    });
    c.rot("body", |t| swing(t, 0.6, 2.0, 0.1));
    let look = Curve::cycle(&[(0.0, 0.0), (0.3, 4.0), (0.55, 4.0), (0.8, -3.0), (1.0, 0.0)]);
    c.rot("neck", |t| look.at(t - 0.03));
    c.rot("head", |t| 0.6 * look.at(t - 0.08));
    let flick = Curve::cycle(&[
        (0.0, 0.0),
        (0.6, 0.0),
        (0.64, -14.0),
        (0.7, 2.0),
        (0.76, 0.0),
        (1.0, 0.0),
    ]);
    trailing(
        &mut c,
        |t, k| swing(t, 3.0 + 2.0 * k as f32, 1.0, 0.1 + 0.07 * k as f32),
        |t| flick.at(t),
    );
    c.done()
}

/// A paw over one cycle, starting at its contact: planted for `stance` of
/// the cycle while it slides back `stride`, lifted `lift` through its
/// swing, both in rig pixels; its paw curls through the swing.
fn stepping(stance: f32, stride: f32, lift: f32, curl: f32) -> (Curve, Curve, Curve) {
    let w = 1.0 - stance;
    let x = Curve::cycle(&[
        (0.0, 0.5 * stride),
        (stance, -0.5 * stride),
        (stance + 0.15 * w, -0.56 * stride),
        (stance + 0.6 * w, 0.1 * stride),
        (stance + 0.85 * w, 0.55 * stride),
        (1.0, 0.5 * stride),
    ]);
    let y = Curve::cycle(&[
        (0.0, 0.0),
        (stance, 0.0),
        (stance + 0.35 * w, lift),
        (stance + 0.7 * w, 0.75 * lift),
        (1.0, 0.0),
    ]);
    let r = Curve::cycle(&[
        (0.0, 4.0),
        (0.1 * stance, 0.0),
        (stance, -18.0 * curl),
        (stance + 0.3 * w, -45.0 * curl),
        (stance + 0.75 * w, -8.0 * curl),
        (1.0, 4.0),
    ]);
    (x, y, r)
}

fn walk(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "walk", 1.1, true);
    let l = m.leg;
    let (x, y, r) = stepping(0.68, 0.62 * l, 0.16 * l, 1.0);
    for (end, side, phase) in WALK {
        let (x, y, r) = (x.clone(), y.clone(), r.clone());
        c.mov(&ik(end, side), move |t| [x.at(t - phase), y.at(t - phase)]);
        c.rot(&paw(end, side), move |t| r.at(t - phase));
    }
    // The body dips as each hind paw lands and rises over the fore; it
    // rocks a little, front up as the fore paws pass under it.
    c.mov("body", |t| {
        [0.0, -0.06 * l + swing(t, 0.012 * l, 2.0, 0.18)]
    });
    c.rot("body", |t| swing(t, 1.4, 2.0, 0.3));
    // The head nods with the fore legs, countering the body.
    c.rot("neck", |t| -2.0 + swing(t, -2.5, 2.0, 0.34));
    c.rot("head", |t| swing(t, 2.0, 2.0, 0.4));
    c.rot("jaw", |_| 0.0);
    trailing(
        &mut c,
        |t, k| -4.0 + swing(t, 4.0 + 2.5 * k as f32, 2.0, 0.12 + 0.08 * k as f32),
        |t| swing(t, 3.0, 2.0, 0.42),
    );
    c.event(0.0, "step");
    c.event(0.25, "step");
    c.event(0.5, "step");
    c.event(0.75, "step");
    c.done()
}

fn run(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "run", 0.5, true);
    let l = m.leg;
    // A bound: the hind pair lands and drives (the far paw first), the
    // body flies gathered, the fore pair lands and pulls, the body flies
    // stretched out.
    let (x, y, r) = stepping(0.36, 0.8 * l, 0.32 * l, 1.2);
    for (end, side, phase) in [
        ("back", "l", 0.0),
        ("back", "r", 0.07),
        ("front", "l", 0.48),
        ("front", "r", 0.55),
    ] {
        let (x, y, r) = (x.clone(), y.clone(), r.clone());
        c.mov(&ik(end, side), move |t| [x.at(t - phase), y.at(t - phase)]);
        c.rot(&paw(end, side), move |t| r.at(t - phase));
    }
    // Front up as the hind legs drive (0.3), front down as the fore paws
    // reach the ground (0.5 to 0.6); highest in each flight.
    let pitch = Curve::cycle(&[
        (0.0, 2.0),
        (0.3, 9.0),
        (0.45, 2.0),
        (0.62, -7.0),
        (0.85, -2.0),
        (1.0, 2.0),
    ]);
    let rise = Curve::cycle(&[
        (0.0, -0.15 * l),
        (0.2, -0.18 * l),
        (0.4, -0.07 * l),
        (0.55, -0.14 * l),
        (0.7, -0.17 * l),
        (0.9, -0.06 * l),
        (1.0, -0.15 * l),
    ]);
    c.rot("body", |t| pitch.at(t));
    c.mov("body", |t| [0.0, rise.at(t)]);
    // The head reaches forwards and stays level against the body.
    c.rot("neck", |t| -8.0 - 0.7 * pitch.at(t - 0.04));
    c.rot("head", |t| 4.0 - 0.3 * pitch.at(t - 0.08));
    trailing(
        &mut c,
        |t, k| -16.0 - 4.0 * k as f32 - 1.5 * pitch.at(t - 0.06 * (k + 1) as f32),
        |t| -18.0 + 0.6 * pitch.at(t - 0.05),
    );
    c.event(0.0, "step");
    c.event(0.48, "step");
    c.done()
}

/// Where the legs hang from the body, rig pixels from the body's origin
/// along x: the fore legs ahead (`front`), the hind behind (`back`, as a
/// distance). Pitching the body about its origin raises one end and
/// lowers the other by these times the sine.
struct Frame {
    front: f32,
    back: f32,
}

impl Frame {
    fn new(rig: &Rig2dAsset, m: &Measure) -> Frame {
        let o = origin(rig, "body").map_or(0.0, |o| o[0]);
        let x = |n: &str| origin(rig, n).map(|p| p[0] - o);
        Frame {
            front: x("leg-front-upper-r")
                .or(x("leg-front-upper-l"))
                .unwrap_or(0.6 * m.leg),
            back: -x("leg-back-upper-r")
                .or(x("leg-back-upper-l"))
                .unwrap_or(-0.6 * m.leg),
        }
    }

    /// How far to move the body up as it pitches `deg` so the hind hips
    /// stay where they were (a play bow).
    fn about_back(&self, deg: f32) -> f32 {
        self.back * deg.to_radians().sin()
    }

    /// The same, keeping the fore shoulders where they were (sitting).
    fn about_front(&self, deg: f32) -> f32 {
        -self.front * deg.to_radians().sin()
    }
}

fn jump(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "jump", 1.2, false);
    let l = m.leg;
    let frame = Frame::new(rig, m);
    // Crouched on the haunches (0.2), the hind legs drive (0.3), flying
    // stretched out (0.45 to 0.62), the fore paws reach down (0.72) and
    // land (0.78), then the hind (0.84), absorbing.
    let k = [0.0, 0.2, 0.3, 0.45, 0.62, 0.72, 0.78, 0.84, 0.92, 1.0];
    let key = |v: [f32; 10]| Curve::keyed(&k, &v);
    let pitch = key([0.0, 8.0, 18.0, 4.0, -6.0, -14.0, -8.0, -2.0, 0.0, 0.0]);
    // The body's lift, its hind hips kept down through the drive.
    let lift = key([0.0, -0.24, -0.1, 0.5, 0.52, 0.26, -0.08, -0.14, -0.05, 0.0].map(|v| v * l));
    // Each pair's paws relative to the body: tucked up (positive) or
    // reaching (negative), on the ground wherever that is below it.
    let fore_x = key([0.0, 0.0, -0.08, 0.26, 0.3, 0.2, 0.0, 0.0, 0.0, 0.0].map(|v| v * l));
    let fore_up = key([0.0, 0.0, 0.18, 0.3, 0.2, 0.06, 0.0, 0.0, 0.0, 0.0].map(|v| v * l));
    let hind_x = key([0.0, 0.0, 0.0, -0.24, -0.2, 0.0, 0.05, 0.0, 0.0, 0.0].map(|v| v * l));
    let hind_up = key([0.0, 0.0, 0.0, 0.18, 0.24, 0.3, 0.14, 0.0, 0.0, 0.0].map(|v| v * l));
    let curl = key([0.0, 0.0, -20.0, -40.0, -20.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let (front, back) = (frame.front, frame.back);
    c.rot("body", |t| pitch.at(t));
    c.mov("body", |t| [0.0, lift.at(t)]);
    for side in ["l", "r"] {
        let lead = if side == "l" { 0.015 } else { 0.0 };
        let (fx, fu, hx, hu) = (
            fore_x.clone(),
            fore_up.clone(),
            hind_x.clone(),
            hind_up.clone(),
        );
        let (p1, p2, l1, l2) = (pitch.clone(), pitch.clone(), lift.clone(), lift.clone());
        // Each end of the body rises with the lift and its pitch.
        c.mov(&ik("front", side), move |t| {
            let t = t - lead;
            let end = l1.at(t) + front * p1.at(t).to_radians().sin();
            [fx.at(t), (end + fu.at(t)).max(0.0)]
        });
        c.mov(&ik("back", side), move |t| {
            let t = t - lead;
            let end = l2.at(t) - back * p2.at(t).to_radians().sin();
            [hx.at(t), (end + hu.at(t)).max(0.0)]
        });
        let (c1, c2) = (curl.clone(), curl.clone());
        c.rot(&paw("front", side), move |t| c1.at(t - lead));
        c.rot(&paw("back", side), move |t| c2.at(t - lead - 0.03));
    }
    c.rot("neck", |t| -0.6 * pitch.at(t - 0.04));
    c.rot("head", |t| -0.3 * pitch.at(t - 0.08));
    trailing(
        &mut c,
        |t, k| -1.6 * (1.0 + 0.3 * k as f32) * pitch.at(t - 0.05 * (k + 1) as f32),
        |t| -0.8 * pitch.at(t - 0.06),
    );
    c.event(0.3, "takeoff");
    c.event(0.78, "land");
    c.done()
}

fn attack(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "attack", 0.8, false);
    let l = m.leg;
    // Drawn back low, head down and jaw shut (0.3); lunging forwards, the
    // jaw open (0.42); the bite snapping shut at full reach (0.5); held,
    // shaking; recovering. The fore paws leap ahead and the hind step
    // after them.
    let k = [0.0, 0.3, 0.42, 0.5, 0.62, 1.0];
    let key = |v: [f32; 6]| Curve::keyed(&k, &v);
    let shift = key([0.0, -0.14, 0.18, 0.24, 0.22, 0.0].map(|v| v * l));
    let drop = key([0.0, -0.14, -0.06, -0.08, -0.08, 0.0].map(|v| v * l));
    let pitch = key([0.0, -3.0, 5.0, -2.0, -3.0, 0.0]);
    let neck = key([0.0, -14.0, -6.0, -10.0, -10.0, 0.0]);
    let head = key([0.0, -6.0, 8.0, -6.0, -6.0, 0.0]);
    let jaw = key([0.0, 0.0, -32.0, 0.0, 0.0, 0.0]);
    let step = key([0.0, -0.08, 0.3, 0.32, 0.32, 0.0].map(|v| v * l));
    let raise = key([0.0, 0.0, 0.16, 0.0, 0.0, 0.0].map(|v| v * l));
    c.mov("body", |t| [shift.at(t), drop.at(t)]);
    c.rot("body", |t| pitch.at(t));
    c.rot("neck", |t| neck.at(t - 0.02));
    c.rot("head", |t| {
        head.at(t - 0.04) + swing(t, 3.0, 6.0, 0.0) * (1.0 - ((t - 0.56) / 0.08).abs()).max(0.0)
    });
    c.rot("jaw", |t| jaw.at(t));
    for (side, lag) in [("r", 0.0), ("l", 0.03)] {
        let (s, r) = (step.clone(), raise.clone());
        c.mov(&ik("front", side), move |t| [s.at(t - lag), r.at(t - lag)]);
        let (s, r) = (shift.clone(), raise.clone());
        c.mov(&ik("back", side), move |t| {
            let t = t - lag - 0.08;
            [0.8 * s.at(t).max(0.0), 0.5 * r.at(t)]
        });
    }
    trailing(
        &mut c,
        |t, kk| (1.0 + 0.4 * kk as f32) * (-6.0 + 2.0 * pitch.at(t - 0.05 * (kk + 1) as f32)),
        |t| -1.5 * neck.at(t - 0.04),
    );
    c.event(0.5, "hit");
    c.done()
}

fn hit(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "hit", 0.45, false);
    let l = m.leg;
    // Struck in the face: knocked back, the fore paws skidding, the head
    // thrown up, a flash of red.
    let k = [0.0, 0.18, 0.32, 0.6, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let back = key([0.0, -0.1, -0.08, -0.02, 0.0].map(|v| v * l));
    let pitch = key([0.0, 5.0, 4.0, 1.0, 0.0]);
    let head = key([0.0, 16.0, 12.0, 3.0, 0.0]);
    let frame = Frame::new(rig, m);
    c.mov("body", |t| {
        [
            back.at(t),
            frame.about_front(pitch.at(t)) - 0.06 * l * (back.at(t) / (-0.1 * l)),
        ]
    });
    c.rot("body", |t| pitch.at(t));
    for side in ["l", "r"] {
        let b = back.clone();
        c.mov(&ik("front", side), move |t| [0.7 * b.at(t - 0.02), 0.0]);
    }
    c.rot("neck", |t| head.at(t - 0.03));
    c.rot("head", |t| 0.5 * head.at(t - 0.07));
    trailing(
        &mut c,
        |t, kk| -(1.0 + 0.4 * kk as f32) * 2.0 * pitch.at(t - 0.06 * (kk + 1) as f32),
        |t| -2.0 * head.at(t - 0.05),
    );
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
    // Struck (0.1), the legs give way, hind first (0.4), the body down on
    // the ground (0.6), the head sinking last (0.85), still.
    let floor = slot_box(rig, "body").map_or(0.5 * l, |b| b[1]);
    let k = [0.0, 0.1, 0.4, 0.6, 0.68, 0.85, 1.0];
    let key = |v: [f32; 7]| Curve::keyed(&k, &v);
    let fall = key([0.0, 0.05, 0.4, 1.02, 0.98, 1.0, 1.0].map(|v| v * floor));
    let pitch = key([0.0, 0.0, -6.0, 0.0, 1.0, 0.0, 0.0]);
    let neck = key([0.0, 12.0, -4.0, -12.0, -16.0, -34.0, -32.0]);
    let head = key([0.0, 8.0, 0.0, -6.0, -6.0, -14.0, -14.0]);
    // Lying down, the fore legs stretch out in front along the ground
    // (their knees bend backwards from the start, where they are
    // straight, so none pokes up) and the hind legs reach back, so no
    // knee bends shut.
    let fore = key([0.0, 0.0, 0.2, 0.9, 1.0, 1.0, 1.0].map(|v| v * l));
    let hind = key([0.0, 0.0, -0.14, -0.62, -0.7, -0.7, -0.7].map(|v| v * l));
    c.mov("body", |t| [0.0, -fall.at(t)]);
    c.rot("body", |t| pitch.at(t));
    c.rot("neck", |t| neck.at(t - 0.03));
    c.rot("head", |t| head.at(t - 0.06));
    for side in ["l", "r"] {
        c.bend(&ik("front", side), false);
        let (f, h) = (fore.clone(), hind.clone());
        c.mov(&ik("front", side), move |t| [f.at(t), 0.0]);
        c.mov(&ik("back", side), move |t| [h.at(t), 0.0]);
        // The hind paws stay flat as their legs reach back.
        let h = hind.clone();
        c.rot(&paw("back", side), move |t| -55.0 * h.at(t) / l);
    }
    let flop = key([0.0, 6.0, 4.0, -10.0, 4.0, 2.0, 2.0]);
    trailing(
        &mut c,
        |t, kk| (1.0 + 0.4 * kk as f32) * flop.at(t - 0.05 * (kk + 1) as f32),
        |t| 0.8 * flop.at(t - 0.04),
    );
    c.event(0.6, "fall");
    c.done()
}

fn howl(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "howl", 2.0, true);
    let l = m.leg;
    let frame = Frame::new(rig, m);
    // Sitting back on the haunches (the body pitched up about its fore
    // shoulders, the hind end down), the head thrown back and the jaw
    // open, a long call that wavers, a breath, again.
    let k = [0.0, 0.2, 0.75, 0.95, 1.0];
    let key = |v: [f32; 5]| Curve::keyed(&k, &v);
    let call = key([0.0, 1.0, 1.0, 0.0, 0.0]);
    let sit = |t: f32| 16.0 + 3.0 * call.at(t);
    c.rot("body", sit);
    c.mov("body", |t| {
        [-0.06 * l, frame.about_front(sit(t)) - 0.02 * l]
    });
    for side in ["l", "r"] {
        c.mov(&ik("back", side), move |_| [0.12 * l, 0.0]);
    }
    c.rot("neck", |t| 18.0 + 22.0 * call.at(t - 0.02));
    c.rot("head", |t| {
        6.0 + 16.0 * call.at(t - 0.05) + swing(t, 2.0, 6.0, 0.0) * call.at(t)
    });
    c.rot("jaw", |t| {
        -24.0 * call.at(t - 0.04) + swing(t, 3.0, 6.0, 0.02) * call.at(t)
    });
    trailing(
        &mut c,
        |t, kk| 4.0 + swing(t, 3.0 + kk as f32, 1.0, 0.1 * (kk + 1) as f32),
        |t| -20.0 * call.at(t - 0.04),
    );
    c.done()
}

fn play(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "play", 1.2, true);
    let l = m.leg;
    let frame = Frame::new(rig, m);
    // A play bow: the chest dropped between the fore legs (the body
    // pitched down about its hind hips), the rump up, the tail wagging
    // hard, bouncing on the fore paws.
    let bow = |t: f32| -14.0 + 2.0 * (0.5 + 0.5 * (std::f32::consts::TAU * 2.0 * t).cos());
    c.rot("body", bow);
    c.mov("body", |t| [0.0, frame.about_back(bow(t)) - 0.07 * l]);
    for (side, lag) in [("r", 0.0), ("l", 0.04)] {
        c.mov(&ik("front", side), move |t| {
            [
                0.14 * l,
                (0.06 * l * (std::f32::consts::TAU * 2.0 * (t - lag)).sin()).max(0.0),
            ]
        });
    }
    c.rot("neck", |t| 16.0 + swing(t, 4.0, 2.0, 0.06));
    c.rot("head", |t| 8.0 + swing(t, 8.0, 1.0, 0.1));
    trailing(
        &mut c,
        |t, kk| 18.0 + swing(t, 20.0 + 8.0 * kk as f32, 6.0, 0.03 * kk as f32),
        |t| swing(t, 6.0, 2.0, 0.1),
    );
    c.done()
}

fn sniff(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "sniff", 1.6, true);
    let l = m.leg;
    let frame = Frame::new(rig, m);
    // The nose down to the ground, quick small sniffs, a step forwards
    // with a fore paw, the tail raised.
    let down = Curve::cycle(&[(0.0, 0.6), (0.15, 1.0), (0.6, 1.0), (0.8, 0.5), (1.0, 0.6)]);
    let lean = |t: f32| -4.0 * down.at(t);
    c.rot("neck", |t| -30.0 * down.at(t));
    c.rot("head", |t| {
        -18.0 * down.at(t - 0.04) + swing(t, 2.5, 8.0, 0.0) * down.at(t)
    });
    c.rot("body", lean);
    c.mov("body", |t| [0.0, frame.about_back(lean(t)) - 0.03 * l]);
    let step = Curve::cycle(&[(0.0, 0.0), (0.6, 0.0), (0.7, 0.1), (0.8, 0.0), (1.0, 0.0)]);
    let reach = Curve::cycle(&[(0.0, 0.0), (0.6, 0.0), (0.8, 0.14), (0.95, 0.0), (1.0, 0.0)]);
    c.mov(&ik("front", "r"), move |t| {
        [reach.at(t) * l, step.at(t) * l]
    });
    trailing(
        &mut c,
        |t, kk| 10.0 + swing(t, 4.0 + 2.0 * kk as f32, 2.0, 0.08 * kk as f32),
        |t| -6.0 + swing(t, 4.0, 4.0, 0.0),
    );
    c.done()
}

/// Every quadruped clip, in the library's order.
pub fn clips(rig: &Rig2dAsset, m: &Measure) -> Vec<Clip2d> {
    vec![
        idle(rig, m),
        walk(rig, m),
        run(rig, m),
        jump(rig, m),
        attack(rig, m),
        hit(rig, m),
        die(rig, m),
        howl(rig, m),
        play(rig, m),
        sniff(rig, m),
    ]
}
