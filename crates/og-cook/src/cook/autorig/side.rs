//! The humanoid-side clips: idle, walk, run, jump, attack, hit, die and
//! the emotes wave, cheer and nod, for a character facing right. Feet
//! are placed by the legs' IK targets (`ik-foot-l`, `ik-foot-r`) and the
//! hips carry the body, so a walk plants each foot on the ground from
//! contact to toe-off while the hips dip and rise over it.

use super::clips::{Clip, Curve, Measure, swing};
use og_anim::{Clip2d, Rig2dAsset};

/// The far (`l`) and near (`r`) sides, and the half cycle between them.
const SIDES: [(&str, f32); 2] = [("r", 0.0), ("l", 0.5)];

/// Whether the hand on `side` holds the weapon or the shield: that arm
/// swings less and its hand steadies what it holds.
fn holds(rig: &Rig2dAsset, side: &str) -> bool {
    let hand = rig.bone(&format!("hand-{side}"));
    ["weapon", "shield"]
        .iter()
        .filter_map(|n| rig.bone(n))
        .any(|b| rig.bones[b].parent == hand)
}

/// The free hand: the near one unless it holds something.
fn free_side(rig: &Rig2dAsset) -> &'static str {
    if holds(rig, "r") && !holds(rig, "l") {
        "l"
    } else {
        "r"
    }
}

/// An arm on `side`: the upper arm, the forearm and the hand, each a
/// function of the clip's fraction. A holding hand turns back by part of
/// what the arm turns, so a sword keeps its line instead of waving.
fn arm(
    c: &mut Clip,
    rig: &Rig2dAsset,
    side: &str,
    upper: impl Fn(f32) -> f32,
    lower: impl Fn(f32) -> f32,
    hand: impl Fn(f32) -> f32,
) {
    let steady = holds(rig, side);
    c.rot(&format!("arm-upper-{side}"), &upper);
    c.rot(&format!("arm-lower-{side}"), &lower);
    c.rot(&format!("hand-{side}"), |t| {
        if steady {
            -0.7 * (upper(t) + lower(t))
        } else {
            hand(t)
        }
    });
}

fn chains(c: &mut Clip, f: impl Fn(f32, usize, f32) -> f32) {
    // A cape and hair trail the body, a tail too; each bone later and
    // wider than the one before it.
    for (name, scale) in [
        ("cape", 1.0),
        ("hair", 0.8),
        ("tail", 1.4),
        ("wing-l", 0.5),
        ("wing-r", 0.5),
    ] {
        c.chain(name, |t, k| f(t, k, scale));
    }
}

fn idle(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "idle", 2.4, true);
    let l = m.leg;
    // A breath: the chest rises, the hips settle, the head and arms follow late.
    c.mov("pelvis", |t| {
        [
            0.0,
            -0.012 * l * (1.0 - (std::f32::consts::TAU * t).cos()) / 2.0,
        ]
    });
    // The chest rises with the breath; never by scale, which the head and
    // arms would inherit.
    c.mov("torso", |t| {
        [
            0.0,
            0.01 * l * (0.5 - 0.5 * (std::f32::consts::TAU * (t - 0.08)).cos()),
        ]
    });
    c.rot("torso", |t| swing(t, -0.8, 1.0, 0.1));
    c.rot("neck", |t| swing(t, 0.6, 1.0, 0.2));
    c.rot("head", |t| swing(t, 1.2, 1.0, 0.24));
    for (s, _) in SIDES {
        let sign = if s == "r" { 1.0 } else { -1.0 };
        c.rot(&format!("arm-upper-{s}"), |t| {
            swing(t, 1.5 * sign, 1.0, 0.15)
        });
        c.rot(&format!("arm-lower-{s}"), |t| {
            3.0 + swing(t, 2.0, 1.0, 0.25)
        });
        c.rot(&format!("hand-{s}"), |t| swing(t, 2.0, 1.0, 0.33));
    }
    chains(&mut c, |t, k, s| {
        s * swing(t, 1.5 + k as f32, 1.0, 0.2 + 0.08 * k as f32)
    });
    c.done()
}

fn walk(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "walk", 1.0, true);
    let l = m.leg;
    let s = 0.7 * l;
    // The near foot: contact (0), flat under the body (0.25), toe-off
    // (0.5), passing (0.75) lifted, reaching forwards (0.9).
    let fx = Curve::cycle(&[
        (0.0, 0.5 * s),
        (0.25, 0.0),
        (0.5, -0.5 * s),
        (0.6, -0.42 * s),
        (0.75, 0.0),
        (0.9, 0.44 * s),
        (1.0, 0.5 * s),
    ]);
    let fy = Curve::cycle(&[
        (0.0, 0.0),
        (0.25, 0.0),
        (0.5, 0.0),
        (0.6, 0.09 * l),
        (0.75, 0.15 * l),
        (0.9, 0.06 * l),
        (1.0, 0.0),
    ]);
    // Heel strike toes up, flat, heel rising, the push off the toes, and
    // the foot hanging through the swing.
    let toe = Curve::cycle(&[
        (0.0, 14.0),
        (0.1, 0.0),
        (0.38, -6.0),
        (0.5, -26.0),
        (0.62, -18.0),
        (0.75, -4.0),
        (0.9, 6.0),
        (1.0, 14.0),
    ]);
    // The hips: down just after each contact, up as the leg passes under.
    let hips = Curve::cycle(&[
        (0.0, -0.075 * l),
        (0.2, -0.105 * l),
        (0.5, -0.045 * l),
        (0.72, -0.035 * l),
        (1.0, -0.075 * l),
    ]);
    for (side, phase) in SIDES {
        let (fx, fy, toe) = (fx.clone(), fy.clone(), toe.clone());
        c.mov(&format!("ik-foot-{side}"), move |t| {
            [fx.at(t + phase), fy.at(t + phase)]
        });
        c.rot(&format!("foot-{side}"), move |t| toe.at(t + phase));
    }
    c.mov("pelvis", |t| [0.0, hips.at(2.0 * t)]);
    c.rot("pelvis", |t| swing(t, 1.5, 2.0, 0.05));
    // The torso leans into the walk and counter-rotates against the hips;
    // the head counters the torso, later, to stay level.
    c.rot("torso", |t| -4.0 + swing(t, -1.6, 2.0, 0.09));
    c.rot("neck", |t| 1.5 + swing(t, 0.8, 2.0, 0.15));
    c.rot("head", |t| 3.0 + swing(t, 1.4, 2.0, 0.19));
    // Arms against the legs: the near arm back when the near foot is
    // forward; the forearm and hand trail.
    for (side, phase) in SIDES {
        // A holding arm swings less, its forearm nearly still.
        let k = if holds(rig, side) { 0.4 } else { 1.0 };
        let base = if holds(rig, side) { 4.0 } else { 16.0 };
        arm(
            &mut c,
            rig,
            side,
            move |t| k * swing(t + phase, -20.0, 1.0, 0.0),
            move |t| base + k * swing(t + phase, -11.0, 1.0, 0.08),
            move |t| swing(t + phase, -6.0, 1.0, 0.16),
        );
    }
    chains(&mut c, |t, k, sc| {
        sc * (-3.0 - k as f32 + swing(t, 2.5 + 2.0 * k as f32, 2.0, 0.07 * (k + 1) as f32))
    });
    c.event(0.0, "step");
    c.event(0.5, "step");
    c.done()
}

fn run(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "run", 0.64, true);
    let l = m.leg;
    // The near foot lands under the body (0), drives back to toe-off
    // (0.36), kicks its heel up behind (0.52), swings its knee through
    // (0.72) and reaches to land (0.88).
    let fx = Curve::cycle(&[
        (0.0, 0.25 * l),
        (0.18, -0.08 * l),
        (0.36, -0.42 * l),
        (0.52, -0.46 * l),
        (0.72, 0.05 * l),
        (0.88, 0.3 * l),
        (1.0, 0.25 * l),
    ]);
    // Flat from contact through mid-stance, the heel rising to toe-off.
    let fy = Curve::cycle(&[
        (0.0, 0.0),
        (0.18, 0.0),
        (0.36, 0.05 * l),
        (0.52, 0.42 * l),
        (0.72, 0.36 * l),
        (0.88, 0.1 * l),
        (1.0, 0.0),
    ]);
    let toe = Curve::cycle(&[
        (0.0, 6.0),
        (0.12, 0.0),
        (0.36, -34.0),
        (0.52, -48.0),
        (0.72, -10.0),
        (0.88, 8.0),
        (1.0, 6.0),
    ]);
    // Over one step (half the cycle): lowest at mid-stance, highest in
    // the flight between toe-off (0.72 of the step) and the next contact.
    let hips = Curve::cycle(&[
        (0.0, -0.12 * l),
        (0.36, -0.16 * l),
        (0.72, -0.09 * l),
        (0.86, -0.03 * l),
        (1.0, -0.12 * l),
    ]);
    for (side, phase) in SIDES {
        let (fx, fy, toe) = (fx.clone(), fy.clone(), toe.clone());
        c.mov(&format!("ik-foot-{side}"), move |t| {
            [fx.at(t + phase), fy.at(t + phase)]
        });
        c.rot(&format!("foot-{side}"), move |t| toe.at(t + phase));
    }
    c.mov("pelvis", |t| [0.0, hips.at(2.0 * t)]);
    c.rot("pelvis", |t| swing(t, 2.5, 2.0, 0.04));
    c.rot("torso", |t| -13.0 + swing(t, -2.5, 2.0, 0.08));
    c.rot("neck", |t| 4.0 + swing(t, 1.2, 2.0, 0.14));
    c.rot("head", |t| 7.0 + swing(t, 2.2, 2.0, 0.18));
    // Bent arms pumping hard against the legs.
    for (side, phase) in SIDES {
        // A holding arm pumps less and keeps its blade low and forwards.
        let (k, bend) = if holds(rig, side) {
            (0.5, 30.0)
        } else {
            (1.0, 78.0)
        };
        arm(
            &mut c,
            rig,
            side,
            move |t| 4.0 + k * swing(t + phase, -42.0, 1.0, 0.0),
            move |t| bend + k * swing(t + phase, -16.0, 1.0, 0.07),
            move |t| swing(t + phase, -8.0, 1.0, 0.14),
        );
    }
    // Streaming back in the wind, fluttering.
    chains(&mut c, |t, k, sc| {
        sc * (-14.0 - 3.0 * k as f32 + swing(t, 3.0 + 2.5 * k as f32, 2.0, 0.06 * (k + 1) as f32))
    });
    c.event(0.0, "step");
    c.event(0.5, "step");
    c.done()
}

fn jump(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "jump", 1.1, false);
    let l = m.leg;
    // Anticipation, take-off, the tuck at the top, reaching down, the landing.
    let k = [0.0, 0.18, 0.3, 0.46, 0.66, 0.8, 0.88, 1.0];
    let key = |v: [f32; 8]| Curve::once(&k.iter().copied().zip(v).collect::<Vec<_>>());
    let hips = key([0.0, -0.24, 0.05, 0.02, 0.02, 0.04, -0.2, 0.0].map(|v| v * l));
    let feet = key([0.0, 0.0, 0.0, 0.3, 0.32, 0.06, 0.0, 0.0].map(|v| v * l));
    let tuck = key([0.0, 0.0, 0.0, -0.06, -0.06, 0.02, 0.0, 0.0].map(|v| v * l));
    let toe = key([0.0, 0.0, -32.0, 12.0, 10.0, -14.0, 0.0, 0.0]);
    let torso = key([0.0, -16.0, -4.0, -9.0, -8.0, -4.0, -18.0, 0.0]);
    let head = key([0.0, 12.0, 2.0, 4.0, 5.0, 2.0, 12.0, 0.0]);
    let arm = key([0.0, -38.0, 135.0, 105.0, 95.0, 60.0, 22.0, 0.0]);
    let fore = key([0.0, 18.0, 10.0, 30.0, 30.0, 18.0, 25.0, 0.0]);
    c.mov("pelvis", |t| [0.0, hips.at(t)]);
    for (side, lead) in [("r", 0.0), ("l", 0.02)] {
        let (feet, tuck, toe) = (feet.clone(), tuck.clone(), toe.clone());
        c.mov(&format!("ik-foot-{side}"), move |t| {
            [tuck.at(t - lead), feet.at(t - lead)]
        });
        c.rot(&format!("foot-{side}"), move |t| toe.at(t - lead));
        let (arm, fore) = (arm.clone(), fore.clone());
        let k = if holds(rig, side) { 0.6 } else { 1.0 };
        let upper = move |t: f32| k * arm.at(t - lead);
        let lower = move |t: f32| k * fore.at(t - lead - 0.03);
        self::arm(&mut c, rig, side, upper, lower, |_| 0.0);
    }
    c.rot("torso", |t| torso.at(t));
    c.rot("head", |t| head.at(t - 0.04));
    let trail = key([0.0, 6.0, -26.0, -10.0, 8.0, 18.0, -12.0, 0.0]);
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
    // Wind-up overhead behind (0.3), the strike (0.42), follow-through
    // (0.54), recovery.
    let k = [0.0, 0.3, 0.42, 0.54, 1.0];
    let key = |v: [f32; 5]| Curve::once(&k.iter().copied().zip(v).collect::<Vec<_>>());
    // Raised forwards and up over the head, the blade hanging behind it,
    // then chopped down in front.
    let arm = key([0.0, 172.0, 45.0, 18.0, 0.0]);
    let fore = key([0.0, 18.0, 6.0, 0.0, 0.0]);
    let off = key([0.0, 32.0, -24.0, -30.0, 0.0]);
    // The body leans back under the raised arm so the arm rises behind
    // the crown, clear of the visor.
    let torso = key([0.0, 18.0, -20.0, -24.0, 0.0]);
    let head = key([0.0, -8.0, -7.0, -6.0, 0.0]);
    let hips = key([0.0, -0.03, 0.08, 0.1, 0.0].map(|v| v * l));
    let drop = key([0.0, -0.03, -0.08, -0.09, 0.0].map(|v| v * l));
    let lunge = key([0.0, 0.0, 0.26, 0.26, 0.0].map(|v| v * l));
    let lift = key([0.0, 0.05, 0.0, 0.0, 0.0].map(|v| v * l));
    c.rot("arm-upper-r", |t| arm.at(t));
    c.rot("arm-lower-r", |t| fore.at(t - 0.02));
    c.rot("hand-r", |t| fore.at(t - 0.05) * 0.25);
    c.rot("arm-upper-l", |t| off.at(t));
    c.rot("arm-lower-l", |t| 20.0 * (off.at(t - 0.04) / 32.0).abs());
    c.rot("torso", |t| torso.at(t));
    c.rot("head", |t| head.at(t - 0.04));
    c.mov("pelvis", |t| [hips.at(t), drop.at(t)]);
    c.mov("ik-foot-r", |t| {
        [
            lunge.at(t),
            lift.at(t) + 0.12 * l * (lunge.at(t + 0.05) - lunge.at(t)).abs() / (0.26 * l),
        ]
    });
    c.rot("foot-r", |t| -8.0 * lift.at(t) / (0.05 * l));
    let trail = key([0.0, 10.0, -22.0, -14.0, 0.0]);
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.5 * kk as f32) * trail.at(t - 0.05 * (kk + 1) as f32)
    });
    // The striking arm and what it holds pass over the head: drawn in
    // front of it, so the helmet never hides the blade.
    let side = if holds(rig, "l") && !holds(rig, "r") {
        "l"
    } else {
        "r"
    };
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
    let k = [0.0, 0.18, 0.32, 0.6, 1.0];
    let key = |v: [f32; 5]| Curve::once(&k.iter().copied().zip(v).collect::<Vec<_>>());
    let torso = key([0.0, 15.0, 11.0, 3.0, 0.0]);
    let head = key([0.0, 20.0, 16.0, 4.0, 0.0]);
    let back = key([0.0, -0.07, -0.06, -0.02, 0.0].map(|v| v * l));
    let arms = key([0.0, 28.0, 18.0, 4.0, 0.0]);
    c.rot("torso", |t| torso.at(t));
    c.rot("head", |t| head.at(t - 0.06));
    c.mov("pelvis", |t| [back.at(t), back.at(t) * 0.5]);
    c.rot("arm-upper-r", |t| arms.at(t - 0.03));
    c.rot("arm-upper-l", |t| -arms.at(t - 0.05) * 0.8);
    c.rot("arm-lower-r", |t| arms.at(t - 0.08));
    c.rot("arm-lower-l", |t| arms.at(t - 0.1) * 0.6);
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.4 * kk as f32) * torso.at(t - 0.06 * (kk + 1) as f32)
    });
    // A flash of red that fades.
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
    // Struck (0.12), the knees buckle (0.4), falling back (0.72), the
    // body hits the ground (0.8), a small bounce, still.
    let k = [0.0, 0.12, 0.4, 0.72, 0.8, 0.88, 1.0];
    let key = |v: [f32; 7]| Curve::once(&k.iter().copied().zip(v).collect::<Vec<_>>());
    let fall = key([0.0, 4.0, 14.0, 84.0, 90.0, 87.0, 90.0]);
    // Lying on its back: lifted by how far its back reaches behind the root.
    let lift = key([0.0, 0.0, 0.0, 0.85, 1.0, 1.04, 1.0].map(|v| v * m.back));
    // The knees buckle and straighten again as it lies down.
    let hips = key([0.0, -0.02, -0.3, -0.12, -0.02, -0.03, -0.02].map(|v| v * l));
    let torso = key([0.0, 18.0, -10.0, 6.0, 0.0, 2.0, 0.0]);
    let head = key([0.0, 24.0, -16.0, 14.0, -4.0, 6.0, 4.0]);
    // The arms fly up as it is struck and fall along the body on the ground.
    let arm = key([0.0, 34.0, 14.0, 26.0, 6.0, 10.0, 8.0]);
    let fore = key([0.0, 10.0, 25.0, 30.0, 8.0, 14.0, 12.0]);
    c.rot("root", |t| fall.at(t));
    c.mov("root", |t| [0.0, lift.at(t)]);
    // The feet come down with the body: in the root's space, turned a
    // quarter, down is -x; lying, an ankle rests its own height up.
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
    c.rot("arm-upper-r", |t| arm.at(t - 0.03));
    c.rot("arm-upper-l", |t| arm.at(t - 0.05) * 0.8);
    c.rot("arm-lower-r", |t| fore.at(t - 0.06));
    c.rot("arm-lower-l", |t| fore.at(t - 0.08));
    let flop = key([0.0, 8.0, 4.0, -30.0, 20.0, 6.0, 10.0]);
    chains(&mut c, |t, kk, sc| {
        sc * (1.0 + 0.5 * kk as f32) * flop.at(t - 0.04 * (kk + 1) as f32)
    });
    c.event(0.8, "fall");
    c.done()
}

fn wave(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "wave", 1.5, true);
    let l = m.leg;
    // The free arm raised forwards and up, clear of the head and drawn in
    // front of it, the forearm waving three times; the weight shifts from
    // foot to foot under it and the cape and plume answer.
    let (free, other) = if free_side(rig) == "r" {
        ("r", "l")
    } else {
        ("l", "r")
    };
    c.rot(&format!("arm-upper-{free}"), |t| {
        140.0 + swing(t, 5.0, 3.0, 0.05)
    });
    c.rot(&format!("arm-lower-{free}"), |t| {
        22.0 + swing(t, 28.0, 3.0, 0.0)
    });
    c.rot(&format!("hand-{free}"), |t| swing(t, 16.0, 3.0, 0.04));
    if free == "l" {
        c.in_front(&["arm-upper-l", "arm-lower-l", "hand-l"].map(String::from));
    }
    c.mov("pelvis", |t| {
        [
            swing(t, 0.035 * l, 1.0, 0.0),
            -0.02 * l + swing(t, 0.012 * l, 2.0, 0.0),
        ]
    });
    c.rot("pelvis", |t| swing(t, 2.0, 1.0, 0.04));
    c.rot("torso", |t| {
        3.0 + swing(t, -2.5, 1.0, 0.1) + swing(t, 1.0, 3.0, 0.08)
    });
    c.rot("head", |t| 6.0 + swing(t, 3.0, 1.0, 0.18));
    c.rot(&format!("arm-upper-{other}"), |t| swing(t, 4.0, 1.0, 0.2));
    c.rot(&format!("arm-lower-{other}"), |t| {
        6.0 + swing(t, 4.0, 1.0, 0.28)
    });
    chains(&mut c, |t, k, sc| {
        sc * (swing(t, 3.0 + 2.0 * k as f32, 1.0, 0.12 + 0.07 * k as f32)
            + swing(t, 1.5 + k as f32, 3.0, 0.1 * (k + 1) as f32))
    });
    c.done()
}

fn cheer(rig: &Rig2dAsset, m: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "cheer", 1.0, true);
    let l = m.leg;
    // Both arms up, pumping twice, bouncing on bent knees.
    c.mov("pelvis", |t| {
        [
            0.0,
            -0.05 * l * (0.5 - 0.5 * (std::f32::consts::TAU * 2.0 * t).cos()),
        ]
    });
    c.rot("arm-upper-r", |t| 160.0 + swing(t, 8.0, 2.0, 0.04));
    c.rot("arm-upper-l", |t| 150.0 + swing(t, 8.0, 2.0, 0.07));
    c.rot("arm-lower-r", |t| 14.0 + swing(t, 14.0, 2.0, 0.09));
    c.rot("arm-lower-l", |t| 14.0 + swing(t, 14.0, 2.0, 0.12));
    c.rot("torso", |t| 4.0 + swing(t, 2.0, 2.0, 0.06));
    c.rot("head", |t| 10.0 + swing(t, 3.0, 2.0, 0.12));
    chains(&mut c, |t, k, sc| {
        sc * swing(t, 2.0 + 1.5 * k as f32, 2.0, 0.08 * (k + 1) as f32)
    });
    c.done()
}

fn nod(rig: &Rig2dAsset, _: &Measure) -> Clip2d {
    let mut c = Clip::new(rig, "nod", 1.0, true);
    let head = Curve::cycle(&[
        (0.0, 0.0),
        (0.22, -22.0),
        (0.45, 2.0),
        (0.66, -15.0),
        (0.85, 0.0),
        (1.0, 0.0),
    ]);
    c.rot("head", |t| head.at(t));
    c.rot("neck", |t| head.at(t - 0.03) * 0.35);
    c.rot("torso", |t| head.at(t - 0.06) * 0.12);
    chains(&mut c, |t, k, sc| {
        sc * head.at(t - 0.08 * (k + 1) as f32) * -0.3 * (1.0 + k as f32)
    });
    c.done()
}

/// Every humanoid-side clip, in the library's order.
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
        nod(rig, m),
    ]
}
