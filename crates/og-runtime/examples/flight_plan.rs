//! CPU-only experiment, not a runtime path. Inputs are source-locked plans and
//! recorded poses; outputs are measurements. Never writes a world or an asset.
use og_anim::{HumanoidMap, NONE, SkeletonAsset, Trs};
use og_core::glam::{Quat, Vec3, Vec4};
use og_motion::contract::Pose;
use serde::Deserialize;
use serde_json::json;

/// Production supplies controller position/yaw at 30 Hz. This experiment uses
/// the source hips projected onto Y=0, explicitly not a source elevation track.
#[derive(Clone, Copy, Deserialize)]
struct Root {
    position: [f32; 3],
    yaw: f32,
}

#[derive(Clone, Deserialize)]
struct Contact {
    foot: usize,
    begin: usize,
    end: usize,
    anchor: [f32; 3],
}

#[derive(Clone, Deserialize)]
struct Plan {
    group: Vec<String>,
    clip: String,
    takeoff: usize,
    apex: usize,
    landing: usize,
    recovery: usize,
    up: [f32; 3],
    contacts: Vec<Contact>,
    #[serde(default)]
    root: Vec<Root>,
    /// The simulation's elevation at each sample (the controller's height
    /// above its takeoff ground; docs/design.md §2.7 "Jumps"). The fixture
    /// stands in the source's: its lowest foot above that foot's clearance.
    #[serde(default)]
    elevation: Vec<f32>,
}

#[derive(Deserialize)]
struct Recording {
    plan: Plan,
    positions: Vec<[[f32; 3]; 30]>,
    rotations: Vec<[[[f32; 3]; 3]; 30]>,
    reference: Vec<[[f32; 3]; 30]>,
}

#[derive(Deserialize)]
struct Input {
    plan_sha256: String,
    recordings: Vec<Recording>,
}

fn wrapped(angle: f32) -> f32 {
    og_core::math::atan2(og_core::math::sin(angle), og_core::math::cos(angle))
}

/// A rigid correction curve, not a blend of corrected joint positions. Cubic
/// Hermite with shared central tangents is C1 at every sample (including phase
/// boundaries); nothing resets at takeoff, apex or landing. Angles are unwrapped
/// before interpolation so crossing ±pi cannot spin the body a full turn.
struct RootAlignment(Vec<Vec4>);

impl RootAlignment {
    fn new(generated: &[Root], planned: &[Root]) -> Self {
        assert_eq!(generated.len(), planned.len());
        assert!(generated.len() >= 2);
        let mut points: Vec<Vec4> = Vec::with_capacity(generated.len());
        for (from, to) in generated.iter().zip(planned) {
            let angle = wrapped(to.yaw - from.yaw);
            let angle = points.last().map_or(angle, |p| p.w + wrapped(angle - p.w));
            let turn = og_core::math::quat_from_axis_angle(Vec3::Y, angle);
            let from = Vec3::new(from.position[0], 0.0, from.position[2]);
            let translation = Vec3::from_array(to.position) - turn * from;
            points.push(translation.extend(angle));
        }
        Self(points)
    }

    fn at(&self, sample: f32) -> Vec4 {
        assert!(sample >= 0.0 && sample <= (self.0.len() - 1) as f32);
        let i = (sample as usize).min(self.0.len() - 2);
        let u = sample - i as f32;
        let tangent = |j: usize| {
            let lo = j.saturating_sub(1);
            let hi = (j + 1).min(self.0.len() - 1);
            (self.0[hi] - self.0[lo]) / (hi - lo) as f32
        };
        self.0[i] * ((2.0 * u - 3.0) * u * u + 1.0)
            + tangent(i) * (((u - 2.0) * u + 1.0) * u)
            + self.0[i + 1] * ((-2.0 * u + 3.0) * u * u)
            + tangent(i + 1) * ((u - 1.0) * u * u)
    }

    fn place(&self, sample: f32, root: Trs) -> Trs {
        let offset = self.at(sample);
        let turn = og_core::math::quat_from_axis_angle(Vec3::Y, offset.w);
        Trs {
            translation: turn * root.translation + offset.truncate(),
            rotation: og_anim::unit(turn * root.rotation),
            ..root
        }
    }
}

fn roots(skeleton: &SkeletonAsset, map: &HumanoidMap, poses: &[Vec<Trs>]) -> Vec<Root> {
    poses
        .iter()
        .map(|pose| {
            let world = skeleton.model_space(pose);
            Root {
                position: world[map.node("hips").unwrap() as usize]
                    .translation
                    .to_array(),
                yaw: og_motion::contract::heading(
                    world[map.node("rightUpperLeg").unwrap() as usize]
                        .translation
                        .to_array(),
                    world[map.node("leftUpperLeg").unwrap() as usize]
                        .translation
                        .to_array(),
                ),
            }
        })
        .collect()
}

fn aligned(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    base: &[Vec<Trs>],
    plan: &Plan,
) -> Vec<Vec<Trs>> {
    let alignment = RootAlignment::new(&roots(skeleton, map, base), &plan.root);
    let hips = map.node("hips").unwrap() as usize;
    let parent = skeleton.nodes[hips].parent;
    let output: Vec<_> = base
        .iter()
        .enumerate()
        .map(|(t, p)| {
            let mut pose = p.clone();
            let world = skeleton.model_space(p);
            let placed = alignment.place(t as f32, world[hips]);
            let parent = if parent == NONE {
                Trs::IDENTITY
            } else {
                world[parent as usize]
            };
            pose[hips].translation = parent.inverse().transform_point(placed.translation);
            pose[hips].rotation = og_anim::unit(parent.rotation.conjugate() * placed.rotation);
            pose
        })
        .collect();
    for t in (0..base.len()).rev().chain((0..base.len()).step_by(7)) {
        let world = skeleton.model_space(&base[t]);
        let placed = alignment.place(t as f32, world[hips]);
        let parent = if parent == NONE {
            Trs::IDENTITY
        } else {
            world[parent as usize]
        };
        assert_eq!(
            output[t][hips].translation,
            parent.inverse().transform_point(placed.translation)
        );
        assert_eq!(
            output[t][hips].rotation,
            og_anim::unit(parent.rotation.conjugate() * placed.rotation)
        );
    }
    output
}

/// The generated clip on the plan's flight (docs/design.md §2.7, "Jumps"):
/// retimed so its own takeoff and touchdown (`flight`) fall on the plan's,
/// its hips at the plan's elevation plus the pose's local height
/// (`og_motion::flight`). The plan's flight phase is its time fraction,
/// which is the controller's ballistic phase (v0 − v)/(2 v0) exactly.
fn on_flight(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    base: &[Vec<Trs>],
    plan: &Plan,
    flight: og_motion::flight::ClipFlight,
) -> Vec<Vec<Trs>> {
    use og_motion::flight::{JumpTime, local_height, retime};
    let hips = map.node("hips").unwrap() as usize;
    let feet = [
        map.node("leftFoot").unwrap() as usize,
        map.node("rightFoot").unwrap() as usize,
    ];
    let lowest = |p: &[Trs]| {
        let w = skeleton.model_space(p);
        w[feet[0]].translation.y.min(w[feet[1]].translation.y)
    };
    let clearance = base.iter().map(|p| lowest(p)).fold(f32::MAX, f32::min);
    let parent = skeleton.nodes[hips].parent;
    (0..base.len())
        .map(|t| {
            let time = if t < plan.takeoff {
                JumpTime::Prepare {
                    elapsed: t as f32,
                    prepare: plan.takeoff as f32,
                }
            } else if t < plan.landing {
                JumpTime::Air {
                    phase: (t - plan.takeoff) as f32 / (plan.landing - plan.takeoff) as f32,
                }
            } else {
                JumpTime::Landed {
                    since: (t - plan.landing) as f32,
                }
            };
            let s = retime(flight, 0, base.len(), 1.0, time);
            let (i, u) = (s as usize, s.fract());
            let j = (i + 1).min(base.len() - 1);
            let mut pose: Vec<Trs> = base[i]
                .iter()
                .zip(&base[j])
                .map(|(a, b)| a.lerp(b, u))
                .collect();
            let world = skeleton.model_space(&pose);
            let y = plan.elevation[t]
                + local_height(world[hips].translation.y, lowest(&pose), clearance);
            let at = Vec3::new(world[hips].translation.x, y, world[hips].translation.z);
            let parent = if parent == NONE {
                Trs::IDENTITY
            } else {
                world[parent as usize]
            };
            pose[hips].translation = parent.inverse().transform_point(at);
            pose
        })
        .collect()
}

/// A clip's own flight by the plan's screen (no foot below 0.15 m of
/// `scale`), from the first sample of the plan's lead-in.
fn screened_flight(positions: &[[[f32; 3]; 30]], scale: f32) -> og_motion::flight::ClipFlight {
    let contacts: Vec<[f32; 4]> = positions
        .iter()
        .map(|p| {
            let down = |j: usize| if p[j][1] < 0.15 * scale { 1.0 } else { 0.0 };
            [down(24), down(24), down(28), down(28)]
        })
        .collect();
    og_motion::flight::clip_flight(&contacts, 1, 1).expect("a recorded jump leaves the ground")
}

/// Model-quality diagnostics only. The full source hips Y is retained here,
/// unlike the ground-projected root used by the operator.
fn drift(generated: &[Root], source: &[Root]) -> serde_json::Value {
    let samples: Vec<_> = generated.iter().zip(source).enumerate().map(|(sample, (g, s))| {
        json!({"sample":sample, "seconds":sample as f32 / 30.0,
            "horizontal_m":Vec3::new(g.position[0]-s.position[0],0.0,g.position[2]-s.position[2]).length(),
            "yaw_degrees":wrapped(g.yaw-s.yaw).to_degrees(),
            "vertical_m":g.position[1]-s.position[1]})
    }).collect();
    let peak = |key: &str| {
        samples
            .iter()
            .max_by(|a, b| {
                a[key]
                    .as_f64()
                    .unwrap()
                    .abs()
                    .total_cmp(&b[key].as_f64().unwrap().abs())
            })
            .unwrap()
            .clone()
    };
    json!({"scope":"model quality only; not operator acceptance", "samples":samples,
        "horizontal_max":peak("horizontal_m"), "yaw_absolute_max":peak("yaw_degrees"),
        "vertical_absolute_max":peak("vertical_m")})
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn weight(plan: &Plan, sample: usize) -> f32 {
    if sample < plan.takeoff {
        smooth(sample as f32 / plan.takeoff as f32)
    } else if sample <= plan.landing {
        1.0
    } else if sample >= plan.recovery {
        0.0
    } else {
        smooth((plan.recovery - sample) as f32 / (plan.recovery - plan.landing) as f32)
    }
}

fn lean(a: Vec3, b: Vec3, up: Vec3) -> f32 {
    let d = (b - a).normalize();
    og_core::math::atan2(d.cross(up).length(), d.dot(up)).to_degrees()
}

// The whole-clip correction. Continuity is a hard per-joint step bound at
// every sample (the recorded raw-root landing bound inside the landing
// window, the raw step outside it), solved over all samples of the clip at
// once in four stages instead of one sample at a time:
//
//  B. pelvis height, the one slack variable: per-sample intervals (the
//     pelvis box, the reach of every hard contact) propagated through the
//     step relation of the hips cluster and a knee proxy, then picked
//     toward the generated height;
//  CK. the legs: foot and knee particles per leg under exact bone lengths,
//     pins and the step bound, free feet pulled toward their planned path;
//  D. the rest of the body: position-based sweeps (upright cone on chest
//     and head, step bound, bone lengths) around the fixed hips cluster,
//     knees and feet, refit to rotations after every sweep.
//
// The rotation refit is the method of Müller, Bender, Chentanez and Macklin,
// "A Robust Method to Extract the Rotational Part of Deformations" (MIG
// 2016), applied per joint to its children's offsets; the position sweeps
// are position-based dynamics projections (Müller et al. 2007). Both are
// algorithms reimplemented here, not ported code.
const SWEEPS: usize = 100;
const PATH_ITERATIONS: usize = 2000;
const MARGIN: f32 = 0.01;
const CONE_DEGREES: f32 = 4.9;
const FIT_ITERATIONS: usize = 4;
const DISTANCE_PASSES: usize = 4;
const KNEE_PROXY: f32 = 0.7;
const REACH_EPSILON: f32 = 0.00001;

struct Leg {
    id: usize,
    hip: usize,
    knee: usize,
    foot: usize,
}

/// The hips and everything below them, by the humanoid map.
struct Body {
    hips: usize,
    chest: usize,
    head: usize,
    legs: [Leg; 2],
    /// The hips and their descendants, parents first.
    order: Vec<usize>,
    children: Vec<Vec<usize>>,
    /// Joints the body sweep never moves: the hips cluster, knees and feet.
    fixed: Vec<bool>,
}

impl Body {
    fn new(skeleton: &SkeletonAsset, map: &HumanoidMap) -> Body {
        let node = |name: &str| map.node(name).unwrap() as usize;
        let hips = node("hips");
        let n = skeleton.nodes.len();
        let mut below = vec![false; n];
        let mut order = Vec::new();
        let mut children = vec![Vec::new(); n];
        for (j, s) in skeleton.nodes.iter().enumerate() {
            if j == hips || (s.parent != NONE && below[s.parent as usize]) {
                below[j] = true;
                order.push(j);
                if j != hips {
                    children[s.parent as usize].push(j);
                }
            }
        }
        let legs = ["left", "right"].map(|side| Leg {
            id: if side == "left" { 24 } else { 28 },
            hip: node(&format!("{side}UpperLeg")),
            knee: node(&format!("{side}LowerLeg")),
            foot: node(&format!("{side}Foot")),
        });
        let mut fixed = vec![false; n];
        fixed[hips] = true;
        for &c in &children[hips] {
            fixed[c] = true;
        }
        for leg in &legs {
            fixed[leg.knee] = true;
            fixed[leg.foot] = true;
        }
        Body {
            hips,
            chest: node("chest"),
            head: node("head"),
            legs,
            order,
            children,
            fixed,
        }
    }
}

fn contact_window(plan: &Plan, contact: &Contact) -> (usize, usize) {
    let approach = if contact.begin >= plan.landing {
        plan.apex
    } else {
        0
    };
    let release = if contact.end <= plan.takeoff {
        plan.takeoff
    } else {
        plan.recovery
    };
    (approach, release)
}

/// The planned foot position at `sample`: the generated foot, approaching
/// the authored anchor during flight before a hard contact (whose schedule
/// never fades or follows generated flags) and leaving it after; `true`
/// while the contact is hard.
fn foot_target(plan: &Plan, generated: Vec3, foot: usize, sample: usize) -> (Vec3, bool) {
    let contact = plan
        .contacts
        .iter()
        .filter(|c| c.foot == foot)
        .filter(|c| {
            let (approach, release) = contact_window(plan, c);
            sample >= approach && sample < release.max(c.end)
        })
        .min_by_key(|c| {
            if (c.begin..c.end).contains(&sample) {
                0
            } else {
                1 + if sample < c.begin {
                    c.begin - sample
                } else {
                    sample - c.end
                }
            }
        });
    let Some(contact) = contact else {
        return (generated, false);
    };
    let (approach, release) = contact_window(plan, contact);
    let anchor = Vec3::from_array(contact.anchor);
    if sample < contact.begin {
        let w = smooth((sample - approach) as f32 / (contact.begin - approach) as f32);
        (generated.lerp(anchor, w), false)
    } else if sample < contact.end {
        (anchor, true)
    } else {
        let w = smooth((release - sample) as f32 / (release - contact.end) as f32);
        (generated.lerp(anchor, w), false)
    }
}

/// Müller 2016: the rotation `r` (from its warm start) maximising
/// Σ (r a_i)·b_i, by at most `iterations` fixed-point steps.
fn extract(a: &[Vec3], b: &[Vec3], mut r: Quat, iterations: usize) -> Quat {
    for _ in 0..iterations {
        let mut numerator = Vec3::ZERO;
        let mut denominator = 0.0;
        for (a, b) in a.iter().zip(b) {
            let ra = r * *a;
            numerator += ra.cross(*b);
            denominator += ra.dot(*b);
        }
        let omega = numerator / (denominator.abs() + 1e-9);
        let angle = omega.length();
        if angle < 1e-10 {
            break;
        }
        r = og_anim::unit(og_core::math::quat_from_axis_angle(omega / angle, angle) * r);
    }
    r
}

/// The perpendicular knee offset of a two-bone leg at span `s`.
fn knee_offset(l1: f32, l2: f32, s: f32) -> f32 {
    let s = s.clamp((l1 - l2).abs() + 1e-6, l1 + l2);
    let a = (s * s + l1 * l1 - l2 * l2) / (2.0 * s);
    (l1 * l1 - a * a).max(0.0).sqrt()
}

/// The span of a two-bone leg whose knee sits `h` off the hip-foot line.
fn span_for_offset(l1: f32, l2: f32, h: f32) -> f32 {
    let h = h.min(l1.min(l2) - 1e-6);
    (l1 * l1 - h * h).sqrt() + (l2 * l2 - h * h).sqrt()
}

fn conflict(
    kind: &str,
    sample: usize,
    deficit: f32,
    detail: serde_json::Value,
) -> serde_json::Value {
    let mut v = json!({"kind": kind, "sample": sample, "deficit_m": deficit});
    if let Some(map) = detail.as_object() {
        for (k, x) in map {
            v[k] = x.clone();
        }
    }
    v
}

/// Corrects the aligned clip `base` under `plan`. `raw_step[t]` is the
/// largest joint step of the recorded raw-root pose into sample `t`, whose
/// landing-window maximum is the acceptance bound. Returns the corrected
/// local poses and every hard-constraint conflict, per sample.
fn correct(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    base: &[Vec<Trs>],
    plan: &Plan,
    raw_step: &[f32],
) -> (Vec<Vec<Trs>>, Vec<serde_json::Value>) {
    let body = Body::new(skeleton, map);
    let n = skeleton.nodes.len();
    let frames = base.len();
    assert_eq!(raw_step.len(), frames);
    let up = Vec3::from_array(plan.up).normalize();
    let hips = body.hips;
    let mut conflicts = Vec::new();
    let names = |j: usize| skeleton.nodes[j].name.clone();
    // The aligned generated pose, model space: positions, rotations and
    // each joint's offset in its parent's rotation frame.
    let world: Vec<Vec<Trs>> = base.iter().map(|p| skeleton.model_space(p)).collect();
    let position: Vec<Vec<Vec3>> = world
        .iter()
        .map(|w| w.iter().map(|x| x.translation).collect())
        .collect();
    let rotation: Vec<Vec<Quat>> = world
        .iter()
        .map(|w| w.iter().map(|x| x.rotation).collect())
        .collect();
    let offset: Vec<Vec<Vec3>> = (0..frames)
        .map(|t| {
            (0..n)
                .map(|j| {
                    let p = skeleton.nodes[j].parent;
                    if j == hips || p == NONE {
                        Vec3::ZERO
                    } else {
                        rotation[t][p as usize].conjugate()
                            * (position[t][j] - position[t][p as usize])
                    }
                })
                .collect()
        })
        .collect();
    // The continuity bound into every sample.
    let window = plan.landing.saturating_sub(2).max(1)..=(plan.landing + 2).min(frames - 1);
    let landing_bound = window.clone().map(|t| raw_step[t]).fold(0.0_f32, f32::max);
    let bound: Vec<f32> = (0..frames)
        .map(|t| {
            if t == 0 {
                0.0
            } else if window.contains(&t) {
                landing_bound * (1.0 - MARGIN)
            } else {
                landing_bound.max(raw_step[t]) * (1.0 - MARGIN)
            }
        })
        .collect();
    // Legs: lengths per sample, planned foot paths, and the pelvis box.
    let l1 = |leg: &Leg, t: usize| offset[t][leg.knee].length();
    let l2 = |leg: &Leg, t: usize| offset[t][leg.foot].length();
    let targets: Vec<Vec<(Vec3, bool)>> = body
        .legs
        .iter()
        .map(|leg| {
            (0..frames)
                .map(|t| foot_target(plan, position[t][leg.foot], leg.id, t))
                .collect()
        })
        .collect();
    let box_of = |t: usize| {
        let leg = l1(&body.legs[0], t) + l2(&body.legs[0], t);
        (-0.25 * leg, 0.10 * leg)
    };
    let y0: Vec<f32> = (0..frames).map(|t| position[t][hips].y).collect();
    let rel = |t: usize, j: usize| position[t][j] - position[t][hips];
    // The hip joint of `leg` at pelvis height `y`.
    let hip_at = |leg: &Leg, t: usize, y: f32| {
        let r = rel(t, leg.hip);
        Vec3::new(
            position[t][hips].x + r.x,
            y + r.y,
            position[t][hips].z + r.z,
        )
    };

    // ---- B. pelvis height -------------------------------------------------
    let mut lo = vec![0.0_f32; frames];
    let mut hi = vec![0.0_f32; frames];
    let mut pin: Vec<Option<f32>> = vec![None; frames];
    for t in 0..frames {
        let (blo, bhi) = box_of(t);
        lo[t] = y0[t] + blo;
        hi[t] = y0[t] + bhi;
        let mut unreachable = false;
        for (k, leg) in body.legs.iter().enumerate() {
            let (target, hard) = targets[k][t];
            if !hard {
                continue;
            }
            let reach = l1(leg, t) + l2(leg, t) - REACH_EPSILON;
            let d = hip_at(leg, t, y0[t]) - target;
            let square = reach * reach - d.x * d.x - d.z * d.z;
            if square < 0.0 {
                unreachable = true;
                continue;
            }
            let extent = square.sqrt();
            lo[t] = lo[t].max(y0[t] - d.y - extent);
            hi[t] = hi[t].min(y0[t] - d.y + extent);
        }
        if unreachable || lo[t] > hi[t] {
            // No height reaches every hard contact: the height minimising
            // the largest deficit over the box (convex in y), by ternary
            // search; the deficit is the case's reported conflict, the
            // triangle-inequality bound on the contact error under the
            // hard root (the hip joint rigid with the planned pelvis).
            let deficit = |y: f32| {
                body.legs
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| targets[*k][t].1)
                    .map(|(k, leg)| {
                        (hip_at(leg, t, y) - targets[k][t].0).length() - (l1(leg, t) + l2(leg, t))
                    })
                    .fold(f32::MIN, f32::max)
            };
            let (mut a, mut b) = (y0[t] + blo, y0[t] + bhi);
            for _ in 0..100 {
                let m1 = a + (b - a) / 3.0;
                let m2 = b - (b - a) / 3.0;
                if deficit(m1) < deficit(m2) {
                    b = m2;
                } else {
                    a = m1;
                }
            }
            let y = (a + b) * 0.5;
            conflicts.push(conflict(
                "contact reach",
                t,
                deficit(y),
                json!({"pelvis_y": y}),
            ));
            lo[t] = y0[t] + blo;
            hi[t] = y0[t] + bhi;
            pin[t] = Some(y);
        }
    }
    // The step relation of the hips cluster: y_t - y_{t-1} within
    // [shift_lo, shift_hi] keeps every cluster joint's step under the bound.
    let cluster: Vec<usize> = std::iter::once(hips)
        .chain(body.children[hips].iter().copied())
        .collect();
    let mut shift_lo = vec![0.0_f32; frames];
    let mut shift_hi = vec![0.0_f32; frames];
    for t in 1..frames {
        let (mut slo, mut shi) = (f32::MIN, f32::MAX);
        for &j in &cluster {
            let d = position[t][j] - position[t - 1][j];
            let dxz = Vec3::new(d.x, 0.0, d.z).length();
            let h = if dxz >= bound[t] {
                conflicts.push(conflict(
                    "root step",
                    t,
                    dxz - bound[t],
                    json!({"joint": names(j)}),
                ));
                0.0
            } else {
                (bound[t] * bound[t] - dxz * dxz).sqrt()
            };
            let c = d.y - (y0[t] - y0[t - 1]);
            slo = slo.max(-c - h);
            shi = shi.min(-c + h);
        }
        if slo > shi {
            conflicts.push(conflict("root step", t, slo - shi, json!({})));
            slo = (slo + shi) * 0.5;
            shi = slo;
        }
        shift_lo[t] = slo;
        shift_hi[t] = shi;
    }
    // [lo, hi] of y_t compatible with y_{t-1}: the cluster step, and for a
    // foot pinned at both samples the knee proxy (its perpendicular offset
    // moving at most KNEE_PROXY of the bound; span rises with the hip).
    let next_interval = |t: usize, y_prev: f32| -> (f32, f32) {
        let (mut a, mut b) = (y_prev + shift_lo[t], y_prev + shift_hi[t]);
        for (k, leg) in body.legs.iter().enumerate() {
            let ((t0, hard0), (t1, hard1)) = (targets[k][t - 1], targets[k][t]);
            if !(hard0 && hard1) {
                continue;
            }
            let (l1, l2) = (l1(leg, t), l2(leg, t));
            let h0 = knee_offset(l1, l2, (hip_at(leg, t - 1, y_prev) - t0).length());
            let hb = bound[t] * KNEE_PROXY;
            let r = rel(t, leg.hip);
            let hj = position[t][leg.hip];
            let dxz2 = (hj.x - t1.x) * (hj.x - t1.x) + (hj.z - t1.z) * (hj.z - t1.z);
            let y_for_h = |h: f32| {
                let s = span_for_offset(l1, l2, h.max(0.0));
                (s * s >= dxz2).then(|| t1.y - r.y + (s * s - dxz2).sqrt())
            };
            if let Some(y) = y_for_h(h0 + hb) {
                a = a.max(y);
            }
            if h0 - hb > 0.0
                && let Some(y) = y_for_h(h0 - hb)
            {
                b = b.min(y);
            }
        }
        (a, b)
    };
    // [lo, hi] of y_{t-1} within [l0, h0] that can reach y_t = y_next; the
    // interval ends are nondecreasing in y_{t-1}, so bisection finds them.
    let prev_interval = |t: usize, y_next: f32, l0: f32, h0: f32| -> Option<(f32, f32)> {
        if next_interval(t, h0).1 < y_next {
            return None;
        }
        let left = if next_interval(t, l0).1 >= y_next {
            l0
        } else {
            let (mut a, mut b) = (l0, h0);
            for _ in 0..60 {
                let m = (a + b) * 0.5;
                if next_interval(t, m).1 >= y_next {
                    b = m;
                } else {
                    a = m;
                }
            }
            b
        };
        if next_interval(t, l0).0 > y_next {
            return None;
        }
        let right = if next_interval(t, h0).0 <= y_next {
            h0
        } else {
            let (mut a, mut b) = (l0, h0);
            for _ in 0..60 {
                let m = (a + b) * 0.5;
                if next_interval(t, m).0 <= y_next {
                    a = m;
                } else {
                    b = m;
                }
            }
            a
        };
        (left <= right).then_some((left, right))
    };
    for t in 1..frames {
        let a0 = next_interval(t, lo[t - 1]).0;
        let b1 = next_interval(t, hi[t - 1]).1;
        let (mut nlo, mut nhi) = (lo[t].max(a0), hi[t].min(b1));
        if nlo > nhi {
            conflicts.push(conflict("pelvis step", t, nlo - nhi, json!({})));
            let y = match pin[t] {
                Some(y) => y,
                None => ((nlo + nhi) * 0.5).clamp(lo[t], hi[t]),
            };
            nlo = y;
            nhi = y;
        }
        lo[t] = nlo;
        hi[t] = nhi;
    }
    for t in (1..frames).rev() {
        let r0 = prev_interval(t, lo[t], lo[t - 1], hi[t - 1]);
        let r1 = prev_interval(t, hi[t], lo[t - 1], hi[t - 1]);
        let both: Vec<_> = [r0, r1].into_iter().flatten().collect();
        if both.is_empty() {
            conflicts.push(conflict("pelvis step (backward)", t, 0.0, json!({})));
            continue;
        }
        let a = both.iter().map(|r| r.0).fold(f32::MAX, f32::min);
        let b = both.iter().map(|r| r.1).fold(f32::MIN, f32::max);
        let (nlo, nhi) = (lo[t - 1].max(a), hi[t - 1].min(b));
        if nlo > nhi {
            conflicts.push(conflict(
                "pelvis step (backward)",
                t - 1,
                nlo - nhi,
                json!({}),
            ));
            continue;
        }
        lo[t - 1] = nlo;
        hi[t - 1] = nhi;
    }
    // Pick the heights: conflict samples first (a contact outranks the
    // generated pose), then everything toward the generated height, the
    // intervals staying hard.
    let pick = |y: &mut [f32], target: &[f32], weight: &[f32], lo: &[f32], hi: &[f32]| {
        for it in 0..PATH_ITERATIONS {
            let pull = (1.0 - it as f32 / (0.6 * PATH_ITERATIONS as f32)).max(0.0) * 0.3;
            if pull > 0.0 {
                for t in 0..frames {
                    let d = target[t] - y[t];
                    y[t] += pull * weight[t] * d;
                }
            }
            for t in 0..frames {
                y[t] = y[t].clamp(lo[t], hi[t]);
            }
            for t in 1..frames {
                let (a, b) = next_interval(t, y[t - 1]);
                if y[t] > b {
                    let e = y[t] - b;
                    y[t] -= e * 0.5;
                    y[t - 1] += e * 0.5;
                } else if y[t] < a {
                    let e = a - y[t];
                    y[t] += e * 0.5;
                    y[t - 1] -= e * 0.5;
                }
            }
            for t in 0..frames {
                y[t] = y[t].clamp(lo[t], hi[t]);
            }
        }
    };
    let target: Vec<f32> = (0..frames).map(|t| pin[t].unwrap_or(y0[t])).collect();
    let pinned: Vec<f32> = (0..frames)
        .map(|t| if pin[t].is_some() { 1.0 } else { 0.0 })
        .collect();
    let mut y: Vec<f32> = (0..frames).map(|t| y0[t].clamp(lo[t], hi[t])).collect();
    if pin.iter().any(Option::is_some) {
        pick(&mut y, &target, &pinned, &lo, &hi);
        for t in 0..frames {
            if pin[t].is_some() {
                lo[t] = y[t];
                hi[t] = y[t];
            }
        }
    }
    pick(&mut y, &target, &vec![1.0; frames], &lo, &hi);
    for t in 1..frames {
        let (a, b) = next_interval(t, y[t - 1]);
        if y[t] > b + 1e-6 || y[t] < a - 1e-6 {
            conflicts.push(conflict(
                "pelvis pick",
                t,
                (y[t] - b).max(a - y[t]),
                json!({}),
            ));
        }
    }
    let hips_at: Vec<Vec3> = (0..frames)
        .map(|t| Vec3::new(position[t][hips].x, y[t], position[t][hips].z))
        .collect();

    // ---- CK. the legs ----------------------------------------------------------
    let mut feet: Vec<Vec<Vec3>> = Vec::new();
    let mut knees: Vec<Vec<Vec3>> = Vec::new();
    for (k, leg) in body.legs.iter().enumerate() {
        let hip: Vec<Vec3> = (0..frames).map(|t| hip_at(leg, t, y[t])).collect();
        let pinned: Vec<bool> = (0..frames).map(|t| targets[k][t].1).collect();
        let mut path: Vec<Vec3> = (0..frames).map(|t| targets[k][t].0).collect();
        for t in 0..frames {
            let reach = l1(leg, t) + l2(leg, t) - REACH_EPSILON;
            let d = path[t] - hip[t];
            let n = d.length();
            if n > reach {
                path[t] = hip[t] + d / n * reach;
            }
        }
        let goal = path.clone();
        // The knee seed: the generated knee projected onto its circle.
        let mut knee: Vec<Vec3> = (0..frames)
            .map(|t| {
                let (l1, l2) = (l1(leg, t), l2(leg, t));
                let d = path[t] - hip[t];
                let s = d.length();
                let dh = d / s;
                let a = (s * s + l1 * l1 - l2 * l2) / (2.0 * s);
                let h = (l1 * l1 - a * a).max(0.0).sqrt();
                let centre = hip[t] + dh * a;
                let mut g = position[t][leg.knee] - centre;
                g -= dh * g.dot(dh);
                if g.length() < 1e-6 {
                    g = dh.cross(up);
                }
                if g.length() < 1e-6 {
                    g = dh.cross(Vec3::X);
                }
                centre + g.normalize() * h
            })
            .collect();
        let lengths = |t: usize| (l1(leg, t), l2(leg, t));
        let settle = |t: usize, path: &mut [Vec3], knee: &mut [Vec3]| {
            let (l1, l2) = lengths(t);
            let d = knee[t] - hip[t];
            knee[t] = hip[t] + d / d.length() * l1;
            let d = path[t] - knee[t];
            let n = d.length();
            let correction = (n - l2) * d / n;
            if pinned[t] {
                knee[t] += correction;
            } else {
                knee[t] += correction * 0.5;
                path[t] -= correction * 0.5;
            }
            let d = knee[t] - hip[t];
            knee[t] = hip[t] + d / d.length() * l1;
        };
        for it in 0..PATH_ITERATIONS {
            let pull = (1.0 - it as f32 / (0.6 * PATH_ITERATIONS as f32)).max(0.0) * 0.3;
            if pull > 0.0 {
                for t in 0..frames {
                    if !pinned[t] {
                        let d = goal[t] - path[t];
                        path[t] += pull * d;
                    }
                }
            }
            for _ in 0..2 {
                for t in 0..frames {
                    settle(t, &mut path, &mut knee);
                }
                for t in 1..frames {
                    let d = knee[t] - knee[t - 1];
                    let n = d.length();
                    if n > bound[t] {
                        let excess = (n - bound[t]) * d / n;
                        knee[t] -= excess * 0.5;
                        knee[t - 1] += excess * 0.5;
                    }
                    let d = path[t] - path[t - 1];
                    let n = d.length();
                    if n > bound[t] {
                        let excess = (n - bound[t]) * d / n;
                        if pinned[t] && pinned[t - 1] {
                            continue;
                        }
                        if pinned[t - 1] {
                            path[t] -= excess;
                        } else if pinned[t] {
                            path[t - 1] += excess;
                        } else {
                            path[t] -= excess * 0.5;
                            path[t - 1] += excess * 0.5;
                        }
                    }
                }
            }
        }
        // Exact lengths last; the remaining step residuals are reported.
        for t in 0..frames {
            for _ in 0..20 {
                settle(t, &mut path, &mut knee);
            }
            let (l1, l2) = lengths(t);
            let d = knee[t] - hip[t];
            knee[t] = hip[t] + d / d.length() * l1;
            let d = path[t] - knee[t];
            if !pinned[t] {
                path[t] = knee[t] + d / d.length() * l2;
            }
        }
        for t in 1..frames {
            let hard = bound[t] / (1.0 - MARGIN);
            let n = (path[t] - path[t - 1]).length();
            if n > hard + 1e-6 {
                conflicts.push(conflict("foot step", t, n - hard, json!({"foot": leg.id})));
            }
            let n = (knee[t] - knee[t - 1]).length();
            if n > hard + 1e-6 {
                conflicts.push(conflict("knee step", t, n - hard, json!({"foot": leg.id})));
            }
        }
        feet.push(path);
        knees.push(knee);
    }

    // ---- D. the rest of the body ---------------------------------------------
    let cone: Vec<f32> = (0..frames)
        .map(|t| {
            let generated = lean(position[t][hips], position[t][body.chest], up).max(lean(
                position[t][hips],
                position[t][body.head],
                up,
            ));
            let w = weight(plan, t);
            CONE_DEGREES.max((1.0 - w) * generated + w * CONE_DEGREES)
        })
        .collect();
    let bones: Vec<(usize, usize)> = body
        .order
        .iter()
        .filter(|&&j| j != hips)
        .map(|&j| (skeleton.nodes[j].parent as usize, j))
        .collect();
    let mut fit: Vec<Vec<Quat>> = vec![vec![Quat::IDENTITY; n]; frames];
    // FK from the hips over the particle targets `p`: every joint with
    // children takes the rotation that best carries its generated offsets
    // onto them (warm-started), leaves keep their generated rotation.
    let refit = |t: usize, p: &[Vec3], fit: &mut [Quat]| -> (Vec<Vec3>, Vec<Quat>) {
        let mut x = position[t].clone();
        let mut r = rotation[t].clone();
        for &j in &body.order {
            if j == hips {
                x[j] = hips_at[t];
                continue;
            }
            let parent = skeleton.nodes[j].parent as usize;
            x[j] = x[parent] + r[parent] * offset[t][j];
            if body.children[j].is_empty() {
                continue;
            }
            let a: Vec<Vec3> = body.children[j]
                .iter()
                .map(|&c| rotation[t][j] * offset[t][c])
                .collect();
            let b: Vec<Vec3> = body.children[j].iter().map(|&c| p[c] - x[j]).collect();
            fit[j] = extract(&a, &b, fit[j], FIT_ITERATIONS);
            r[j] = og_anim::unit(fit[j] * rotation[t][j]);
        }
        (x, r)
    };
    let legs_into = |t: usize, p: &mut [Vec3]| {
        for (k, leg) in body.legs.iter().enumerate() {
            p[leg.knee] = knees[k][t];
            p[leg.foot] = feet[k][t];
        }
    };
    let mut x: Vec<Vec<Vec3>> = Vec::with_capacity(frames);
    let mut r: Vec<Vec<Quat>> = Vec::with_capacity(frames);
    for t in 0..frames {
        let shift = hips_at[t] - position[t][hips];
        let mut p: Vec<Vec3> = position[t].iter().map(|&q| q + shift).collect();
        legs_into(t, &mut p);
        let (xt, rt) = refit(t, &p, &mut fit[t]);
        x.push(xt);
        r.push(rt);
    }
    for sweep in 0..SWEEPS {
        let mut p = x.clone();
        // The generated pose is the soft objective: a decaying pull toward
        // it (shifted with the pelvis) that ends before the last sweeps, so
        // the hard projections converge exactly from the nearest pose.
        let pull = (1.0 - sweep as f32 / (0.6 * SWEEPS as f32)).max(0.0) * 0.3;
        for t in 0..frames {
            if pull > 0.0 {
                let shift = hips_at[t] - position[t][hips];
                for &j in &body.order {
                    if !body.fixed[j] {
                        let d = position[t][j] + shift - p[t][j];
                        p[t][j] += pull * d;
                    }
                }
            }
            legs_into(t, &mut p[t]);
            for tip in [body.chest, body.head] {
                let v = p[t][tip] - p[t][hips];
                if lean(p[t][hips], p[t][tip], up) > cone[t] {
                    let length = v.length();
                    let perpendicular = v - up * v.dot(up);
                    let (s, c) = og_core::math::sin_cos(cone[t].to_radians());
                    p[t][tip] = p[t][hips] + (up * c + perpendicular.normalize() * s) * length;
                }
            }
        }
        for _ in 0..DISTANCE_PASSES {
            for t in 1..frames {
                for &j in &body.order {
                    if body.fixed[j] {
                        continue;
                    }
                    let d = p[t][j] - p[t - 1][j];
                    let n = d.length();
                    if n <= bound[t] {
                        continue;
                    }
                    let excess = (n - bound[t]) * d / n;
                    p[t][j] -= excess * 0.5;
                    p[t - 1][j] += excess * 0.5;
                }
            }
            for t in 0..frames {
                let project = |p: &mut [Vec3], parent: usize, child: usize| {
                    if body.fixed[child] {
                        return;
                    }
                    let d = p[child] - p[parent];
                    let n = d.length();
                    if n < 1e-12 {
                        return;
                    }
                    let correction = (n - offset[t][child].length()) * d / n;
                    if body.fixed[parent] {
                        p[child] -= correction;
                    } else {
                        p[parent] += correction * 0.5;
                        p[child] -= correction * 0.5;
                    }
                };
                for &(parent, child) in bones.iter().rev() {
                    project(&mut p[t], parent, child);
                }
                for &(parent, child) in &bones {
                    project(&mut p[t], parent, child);
                }
            }
        }
        for t in 0..frames {
            let (xt, rt) = refit(t, &p[t], &mut fit[t]);
            x[t] = xt;
            r[t] = rt;
        }
    }
    // Back to local transforms: only rotations below the hips and the hips'
    // translation change; every other node keeps its aligned pose.
    let parent_of_hips = skeleton.nodes[hips].parent;
    let output = (0..frames)
        .map(|t| {
            let mut pose = base[t].clone();
            let parent = if parent_of_hips == NONE {
                Trs::IDENTITY
            } else {
                world[t][parent_of_hips as usize]
            };
            pose[hips].translation = parent.inverse().transform_point(hips_at[t]);
            for &j in &body.order {
                if j != hips {
                    let p = skeleton.nodes[j].parent as usize;
                    pose[j].rotation = og_anim::unit(r[t][p].conjugate() * r[t][j]);
                }
            }
            let check = skeleton.model_space(&pose);
            for &j in &body.order {
                assert!((check[j].translation - x[t][j]).length() < 1e-4);
            }
            pose
        })
        .collect();
    (output, conflicts)
}

fn measure(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    poses: &[Vec<Trs>],
    plan: &Plan,
    scale: f32,
) -> serde_json::Value {
    let world: Vec<_> = poses.iter().map(|p| skeleton.model_space(p)).collect();
    let hips = map.node("hips").unwrap() as usize;
    let chest = map.node("chest").unwrap() as usize;
    let head = map.node("head").unwrap() as usize;
    let up = Vec3::from_array(plan.up);
    let body = |p: &[Trs]| {
        [
            lean(p[hips].translation, p[chest].translation, up),
            lean(p[hips].translation, p[head].translation, up),
        ]
    };
    let peak = world[plan.takeoff..=plan.landing]
        .iter()
        .flat_map(|p| body(p))
        .fold(0.0_f32, f32::max);
    let mut contact_error = 0.0_f32;
    let mut slide = 0.0_f32;
    let mut contacts = 0;
    for c in &plan.contacts {
        let foot = map
            .node(if c.foot == 24 {
                "leftFoot"
            } else {
                "rightFoot"
            })
            .unwrap() as usize;
        for t in c.begin..c.end {
            contact_error = contact_error
                .max((world[t][foot].translation - Vec3::from_array(c.anchor) * scale).length());
            if t > c.begin {
                slide += (world[t][foot].translation - world[t - 1][foot].translation).length();
                contacts += 1;
            }
        }
    }
    let mut landing_step = 0.0_f32;
    let mut landing_joint = String::new();
    let mut landing_sample = 0;
    for t in plan.landing.saturating_sub(2)..=(plan.landing + 2).min(poses.len() - 1) {
        if t > 0 {
            for (j, (a, b)) in world[t].iter().zip(&world[t - 1]).enumerate() {
                let step = (a.translation - b.translation).length();
                if step > landing_step {
                    landing_step = step;
                    landing_joint = skeleton.nodes[j].name.clone();
                    landing_sample = t;
                }
            }
        }
    }
    json!({"flight_max_degrees": peak, "landing_degrees": body(&world[plan.landing]),
           "apex_degrees": body(&world[plan.apex]), "contact_max_m": contact_error,
           "slide_sum_m": slide, "contact_steps": contacts, "landing_max_step_m": landing_step,
           "landing_max_step_joint": landing_joint, "landing_max_step_sample": landing_sample})
}

fn traveller(path: &str) -> SkeletonAsset {
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: serde_json::Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let nodes = document["nodes"].as_array().unwrap();
    let mut parents = vec![NONE; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        if let Some(children) = node["children"].as_array() {
            for child in children {
                parents[child.as_u64().unwrap() as usize] = i as u32;
            }
        }
    }
    let mut order: Vec<_> = (0..nodes.len()).collect();
    order.sort_by_key(|&i| {
        let mut p = parents[i];
        let mut depth = 0;
        while p != NONE {
            depth += 1;
            p = parents[p as usize];
        }
        depth
    });
    let skeleton = SkeletonAsset {
        nodes: order
            .iter()
            .map(|&i| {
                let node = &nodes[i];
                assert!(node.get("matrix").is_none());
                let vector = |key, default: [f32; 3]| -> [f32; 3] {
                    node.get(key)
                        .map_or(default, |v| serde_json::from_value(v.clone()).unwrap())
                };
                let rotation: [f32; 4] = node.get("rotation").map_or([0., 0., 0., 1.], |v| {
                    serde_json::from_value(v.clone()).unwrap()
                });
                og_anim::SkeletonNode {
                    name: node["name"].as_str().unwrap_or("").into(),
                    parent: if parents[i] == NONE {
                        NONE
                    } else {
                        order
                            .iter()
                            .position(|&p| p == parents[i] as usize)
                            .unwrap() as u32
                    },
                    mesh: NONE,
                    rest: Trs::new(
                        Vec3::from_array(vector("translation", [0.; 3])),
                        Quat::from_array(rotation),
                        Vec3::from_array(vector("scale", [1.; 3])),
                    ),
                }
            })
            .collect(),
        ..Default::default()
    };
    skeleton.validate().unwrap();
    skeleton
}

fn evaluate(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    base: &[Vec<Trs>],
    plan: &Plan,
    rig: &str,
    review: bool,
    flight: Option<og_motion::flight::ClipFlight>,
) -> serde_json::Value {
    // Keep the original raw-root baseline and its continuity bound. Neither
    // re-rooting nor the solve may relax that recorded acceptance bound.
    let before = measure(skeleton, map, base, plan, 1.0);
    let raw_world: Vec<Vec<Trs>> = base.iter().map(|p| skeleton.model_space(p)).collect();
    let raw_step: Vec<f32> = (0..base.len())
        .map(|t| {
            if t == 0 {
                0.0
            } else {
                raw_world[t]
                    .iter()
                    .zip(&raw_world[t - 1])
                    .map(|(a, b)| (a.translation - b.translation).length())
                    .fold(0.0_f32, f32::max)
            }
        })
        .collect();
    // The simulation owns the elevation: the generated clip is drawn in
    // place on the plan's flight before the horizontal root alignment.
    let flown = match flight {
        Some(f) => on_flight(skeleton, map, base, plan, f),
        None => base.to_vec(),
    };
    let placed = aligned(skeleton, map, &flown, plan);
    let aligned_base = measure(skeleton, map, &placed, plan, 1.0);
    let (corrected, conflicts) = correct(skeleton, map, &placed, plan, &raw_step);
    // Random access/seek and sparse draws: the clip is a pure function of
    // its inputs, so a second evaluation read at any sample, in any order,
    // is equal exactly, not within a float tolerance.
    let replay = correct(skeleton, map, &placed, plan, &raw_step).0;
    for t in (0..base.len()).rev().chain((0..base.len()).step_by(7)) {
        assert_eq!(corrected[t], replay[t]);
    }
    let after = measure(skeleton, map, &corrected, plan, 1.0);
    let hips = map.node("hips").unwrap() as usize;
    let pelvis_offset = corrected
        .iter()
        .zip(&placed)
        .map(|(a, b)| {
            (skeleton.model_space(a)[hips].translation - skeleton.model_space(b)[hips].translation)
                .length()
        })
        .fold(0.0_f32, f32::max);
    let deviation: Vec<f32> = corrected
        .iter()
        .zip(&placed)
        .flat_map(|(a, b)| {
            skeleton
                .model_space(a)
                .into_iter()
                .zip(skeleton.model_space(b))
                .map(|(x, y)| (x.translation - y.translation).length())
                .collect::<Vec<_>>()
        })
        .collect();
    let farthest = (0..deviation.len())
        .max_by(|&a, &b| deviation[a].total_cmp(&deviation[b]))
        .unwrap();
    let mut result = json!({"group": plan.group, "clip": plan.clip, "rig": rig,
        "frames": base.len(), "conflicts": conflicts.len(), "conflict_report": conflicts,
        "base": before, "aligned_base": aligned_base, "corrected": after, "pelvis_max_offset_m": pelvis_offset,
        "generated_deviation_max_m": deviation[farthest],
        "generated_deviation_max_joint": skeleton.nodes[farthest % skeleton.nodes.len()].name,
        "generated_deviation_max_sample": farthest / skeleton.nodes.len(),
        "generated_deviation_mean_m": deviation.iter().sum::<f32>() / deviation.len() as f32,
        "landing_step_bound_m": before["landing_max_step_m"],
        "landing_step_pass": after["landing_max_step_m"].as_f64().unwrap() <= before["landing_max_step_m"].as_f64().unwrap()+0.00001,
        "seek_replay_sparse_equal": true});
    if review {
        let positions = |poses: &[Vec<Trs>]| -> Vec<Vec<[f32; 3]>> {
            poses
                .iter()
                .map(|p| {
                    skeleton
                        .model_space(p)
                        .iter()
                        .map(|t| t.translation.to_array())
                        .collect()
                })
                .collect()
        };
        result["review"] = json!({
            "nodes": skeleton.nodes.iter().map(|n| json!({"name": n.name, "parent": n.parent})).collect::<Vec<_>>(),
            "raw": positions(base), "aligned": positions(&placed), "corrected": positions(&corrected),
            "takeoff": plan.takeoff, "apex": plan.apex, "landing": plan.landing, "recovery": plan.recovery,
            "contacts": plan.contacts.iter().map(|c| json!({"begin": c.begin, "end": c.end, "anchor": c.anchor})).collect::<Vec<_>>()
        });
    }
    result
}

fn hash_replay() -> serde_json::Value {
    use og_runtime::*;
    use std::{cell::Cell, rc::Rc};
    struct NoAssets;
    impl Hooks for NoAssets {
        fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
            Ok(String::new())
        }
        fn read_asset(&self, _: &str) -> Fetch<Vec<u8>> {
            Fetch::Failed("CPU fixture has no model assets".into())
        }
    }
    struct CpuPresentation(Rc<Cell<u64>>);
    impl RenderBackend for CpuPresentation {
        fn tier(&self) -> String {
            "none".into()
        }
        fn size(&self) -> (u32, u32) {
            (320, 180)
        }
        fn render(&mut self, _: &og_render::RenderScene) -> Result<(), String> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }
        fn capture(
            &mut self,
            _: &og_render::RenderScene,
            _: u32,
            _: u32,
        ) -> Option<og_render::RgbaImage> {
            None
        }
    }
    fn value(text: String) -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["ok"], true, "{text}");
        v["value"].clone()
    }
    fn start() -> u32 {
        og_host_begin_bundle();
        let game = og_game_game(Some("{\"seed\":11}".into()));
        let w = value(og_engine_create(
            game,
            json!({"root":std::env::temp_dir()}).to_string(),
        ))
        .as_u64()
        .unwrap() as u32;
        let e = value(og_world_spawn(w, None)).as_f64().unwrap();
        for (name, desc) in [
            ("og.Transform", json!({"position":{"x":0,"y":1,"z":0}})),
            ("og.MotionAgent", json!({"drive":true})),
            ("og.Collider", json!({"shape":2,"radius":0.4,"height":1.0})),
            ("og.CharacterController", json!({})),
        ] {
            value(og_world_insert(w, e, name.into(), Some(desc.to_string())));
        }
        let ground = value(og_world_spawn(w, None)).as_f64().unwrap();
        value(og_world_insert(
            w,
            ground,
            "og.Transform".into(),
            Some(json!({"position":{"x":0,"y":-0.5,"z":0}}).to_string()),
        ));
        value(og_world_insert(
            w,
            ground,
            "og.Collider".into(),
            Some(json!({"shape":0,"size":{"x":20,"y":1,"z":20}}).to_string()),
        ));
        value(og_motion_walk_to(
            w,
            e,
            Some(json!({"position":{"x":2.8,"y":0,"z":0}}).to_string()),
        ));
        w
    }
    fn hash(w: u32) -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(&og_engine_probe_dump(w, None)).unwrap();
        v["hash"].clone()
    }
    set_hooks(Rc::new(NoAssets));
    let baseline = start();
    let drawn = start();
    let calls = Rc::new(Cell::new(0));
    let mut snapshot = String::new();
    let mut hashes = Vec::new();
    for tick in 0..120 {
        set_render_backend(
            drawn,
            if tick % 7 == 0 {
                None
            } else {
                Some(Box::new(CpuPresentation(calls.clone())))
            },
        )
        .unwrap();
        value(og_engine_frame(baseline, 1000.0 / 60.0));
        value(og_engine_frame(drawn, 1000.0 / 60.0));
        assert_eq!(hash(baseline), hash(drawn));
        hashes.push(hash(drawn));
        if tick == 59 {
            snapshot = value(og_world_snapshot(drawn, None))
                .as_str()
                .unwrap()
                .into();
        }
    }
    assert!(calls.get() > 0);
    value(og_world_restore(baseline, snapshot.clone(), None));
    value(og_world_restore(drawn, snapshot, None));
    let immediate_equal = hash(drawn) == hashes[59];
    let mut replay_mismatch = None;
    let mut mismatch_ticks = Vec::new();
    for (tick, expected) in hashes.iter().enumerate().skip(60) {
        set_render_backend(drawn, Some(Box::new(CpuPresentation(calls.clone())))).unwrap();
        value(og_engine_frame(baseline, 1000.0 / 60.0));
        value(og_engine_frame(drawn, 1000.0 / 60.0));
        assert_eq!(hash(baseline), hash(drawn));
        if hash(drawn) != *expected {
            mismatch_ticks.push(tick + 1);
        }
        // The first tick after a restore delivers og.WorldRestored
        // (`restored_event.rs`), which the original run never saw: that
        // tick differs by design, and the replay must match from the next.
        if hash(drawn) != *expected && tick > 60 && replay_mismatch.is_none() {
            replay_mismatch = Some(json!({"tick":tick+1,"expected":expected,"actual":hash(drawn)}));
        }
    }
    json!({"ticks":120,"restore_tick":60,"render_callbacks":calls.get(),"hash":hash(drawn),
        "immediate_equal":immediate_equal,"mismatch_ticks":mismatch_ticks,
        "replay_pass":replay_mismatch.is_none(),"replay_mismatch":replay_mismatch,
        "scope":"Controller walk, CPU render callbacks, skipped rendering and snapshot replay; prototype palette not integrated"})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let review = args[5..].iter().any(|arg| arg == "--review");
    // The superseded contract, for comparison: generated pelvis Y kept.
    let generated_height = args[5..].iter().any(|arg| arg == "--generated-height");
    let input: Input = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    let bundle = og_motion::Bundle::from_safetensors(&std::fs::read(&args[2]).unwrap()).unwrap();
    let soma = og_motion::soma::Skeleton::from_bundle(&bundle).unwrap();
    let skeleton = soma.asset();
    let map = HumanoidMap::with_scheme(&skeleton, "soma").unwrap();
    let target = traveller(&args[3]);
    let target_map = og_anim::RigAsset::parse(&std::fs::read_to_string(&args[4]).unwrap())
        .unwrap()
        .map(&target)
        .unwrap();
    let mut results = Vec::new();
    for mut recording in input.recordings {
        assert_eq!(recording.positions.len(), recording.reference.len());
        let source: Vec<_> = recording
            .reference
            .iter()
            .map(|p| Root {
                position: p[0],
                yaw: og_motion::contract::heading(p[26], p[22]),
            })
            .collect();
        let generated: Vec<_> = recording
            .positions
            .iter()
            .map(|p| Root {
                position: p[0],
                yaw: og_motion::contract::heading(p[26], p[22]),
            })
            .collect();
        let model_drift = drift(&generated, &source);
        let lowest = |p: &[[f32; 3]; 30]| p[24][1].min(p[28][1]);
        let clearance = recording
            .reference
            .iter()
            .map(lowest)
            .fold(f32::MAX, f32::min);
        recording.plan.elevation = recording
            .reference
            .iter()
            .map(|p| (lowest(p) - clearance).max(0.0))
            .collect();
        let flight = (!generated_height).then(|| screened_flight(&recording.positions, 1.0));
        recording.plan.root = source
            .iter()
            .map(|r| Root {
                position: [r.position[0], 0.0, r.position[2]],
                yaw: r.yaw,
            })
            .collect();
        let base: Vec<_> = recording
            .positions
            .iter()
            .zip(&recording.rotations)
            .map(|(&positions, &rotations)| {
                soma.local(&Pose {
                    positions,
                    rotations,
                })
            })
            .collect();
        let mut result = evaluate(
            &skeleton,
            &map,
            &base,
            &recording.plan,
            "soma",
            review,
            flight,
        );
        result["generated_flight"] = json!(flight.map(|f| [Some(f.takeoff), f.touchdown]));
        result["model_root_drift_soma"] = model_drift.clone();
        results.push(result);
        let clip = og_motion::soma::clip("recorded", &skeleton, &base);
        let output = og_anim::retarget(
            &skeleton,
            &map,
            &target,
            &target_map,
            &clip,
            &Default::default(),
        )
        .unwrap();
        let binding = og_anim::bind(&target, &output.clip);
        let retargeted: Vec<_> = (0..base.len())
            .map(|t| {
                let mut pose = og_anim::Pose::rest(&target);
                og_anim::sample(
                    &target,
                    &output.clip,
                    &binding,
                    t as f32 / 30.0,
                    NONE,
                    &mut pose,
                );
                pose.nodes
            })
            .collect();
        let mut plan = recording.plan.clone();
        let rest = skeleton.model_space(&skeleton.rest_pose());
        let target_rest = target.model_space(&target.rest_pose());
        let from_hips = rest[map.node("hips").unwrap() as usize].translation;
        let to_hips = target_rest[target_map.node("hips").unwrap() as usize].translation;
        for root in &mut plan.root {
            let p = to_hips + (Vec3::from_array(root.position) - from_hips) * output.scale;
            root.position = [p.x, 0.0, p.z];
        }
        for e in &mut plan.elevation {
            *e *= output.scale;
        }
        for contact in &mut plan.contacts {
            let bone = if contact.foot == 24 {
                "leftFoot"
            } else {
                "rightFoot"
            };
            let from = rest[map.node(bone).unwrap() as usize].translation.y;
            let to = target_rest[target_map.node(bone).unwrap() as usize]
                .translation
                .y;
            let mut anchor = Vec3::from_array(contact.anchor) * output.scale;
            anchor.y += to - from * output.scale;
            contact.anchor = anchor.to_array();
        }
        let mut result = evaluate(
            &target,
            &target_map,
            &retargeted,
            &plan,
            "meshy-traveller",
            review,
            flight,
        );
        result["model_root_drift_soma"] = model_drift;
        results.push(result);
    }
    println!(
        "{}",
        json!({"plan_sha256": input.plan_sha256, "results": results,
        "root_provenance": if generated_height {
            "Ground-projected source hips [x,0,z] and source yaw stand in for controller root; generated pelvis Y unchanged. Horizontal/yaw correction only."
        } else {
            "Ground-projected source hips [x,0,z] and source yaw stand in for controller root; the source's lowest foot above its clearance stands in for controller elevation; the generated clip is retimed onto the plan's flight and drawn in place (og_motion::flight)."
        },
        "original_library_training_jump_windows":0,
        "world_replay": hash_replay(),
        "limitations": ["Prototype palette not integrated", "Gesture priority and leg limits pending",
            "Whole-clip solve: a streaming boundary must hold the clip's samples"]})
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_places_root_without_changing_height_or_internal_pose() {
        let from = Root {
            position: [3.0, 1.7, -2.0],
            yaw: -0.7,
        };
        let to = Root {
            position: [-4.0, 0.0, 6.0],
            yaw: 1.1,
        };
        let alignment = RootAlignment::new(&[from, from], &[to, to]);
        let root = Trs::new(
            Vec3::from_array(from.position),
            og_core::math::quat_from_axis_angle(Vec3::Y, from.yaw),
            Vec3::ONE,
        );
        let child = Trs::new(Vec3::new(0.3, -0.5, 0.1), Quat::IDENTITY, Vec3::ONE);
        let placed = alignment.place(0.0, root);
        assert!((placed.translation - Vec3::new(-4.0, 1.7, 6.0)).length() < 1e-5);
        let forward = placed.rotation * Vec3::Z;
        assert!(
            (forward - Vec3::new(og_core::math::sin(1.1), 0.0, og_core::math::cos(1.1))).length()
                < 1e-5
        );
        assert!(
            (placed.mul(&child).translation.distance(placed.translation)
                - root.mul(&child).translation.distance(root.translation))
            .abs()
                < 1e-5
        );
        assert_eq!(placed.scale, root.scale);
    }

    #[test]
    fn alignment_is_c1_at_phase_samples_and_seek_independent_across_yaw_wrap() {
        let generated = vec![
            Root {
                position: [0.0, 1.3, 0.0],
                yaw: 0.0
            };
            7
        ];
        let planned: Vec<_> = [170.0_f32, 175.0, 179.0, -177.0, -170.0, -173.0, -168.0]
            .iter()
            .enumerate()
            .map(|(i, yaw)| Root {
                position: [i as f32 * 0.2, 0.0, (i as f32 * 0.3).sin()],
                yaw: yaw.to_radians(),
            })
            .collect();
        let alignment = RootAlignment::new(&generated, &planned);
        let pose = Trs::new(Vec3::new(0.3, 1.3, -0.2), Quat::IDENTITY, Vec3::ONE);
        // Nonzero child offset exercises angular as well as linear velocity.
        let point = |s| {
            alignment
                .place(s, pose)
                .mul(&Trs::new(Vec3::X, Quat::IDENTITY, Vec3::ONE))
                .translation
        };
        let eps = 0.001;
        for phase in [1.0, 3.0, 5.0] {
            let left = (point(phase) - point(phase - eps)) / eps;
            let right = (point(phase + eps) - point(phase)) / eps;
            assert!(
                (left - right).length() < 0.003,
                "{phase}: {left:?} != {right:?}"
            );
        }
        let forward: Vec<_> = (0..=24)
            .map(|i| alignment.place(i as f32 / 4.0, pose))
            .collect();
        for i in (0..=24).rev().chain((0..=24).step_by(7)) {
            assert_eq!(alignment.place(i as f32 / 4.0, pose), forward[i]);
        }
        assert!(alignment.at(3.0).w > std::f32::consts::PI);
    }

    #[test]
    fn vertical_drift_retains_sign_and_peak_time_without_changing_alignment() {
        let source = [Root {
            position: [0.0, 1.0, 0.0],
            yaw: 0.0,
        }; 3];
        let mut generated = source;
        generated[0].position[1] = 1.2;
        generated[1].position[1] = 0.4;
        generated[2].position[1] = 1.5;
        let report = drift(&generated, &source);
        assert_eq!(report["vertical_absolute_max"]["sample"], 1);
        assert!(
            (report["vertical_absolute_max"]["vertical_m"]
                .as_f64()
                .unwrap()
                + 0.6)
                .abs()
                < 1e-6
        );
        assert!(
            (report["vertical_absolute_max"]["seconds"].as_f64().unwrap() - 1.0 / 30.0).abs()
                < 1e-6
        );
        assert_eq!(report["samples"].as_array().unwrap().len(), 3);
    }

    /// A straight-legged SOMA-named rig: hips, a four-joint spine to the
    /// head, two legs of equal 0.42 m bones with a toe. Legs are vertical,
    /// so a planted foot at full reach is exactly straight.
    fn rig() -> (SkeletonAsset, HumanoidMap) {
        let node = |name: &str, parent: u32, offset: [f32; 3]| og_anim::SkeletonNode {
            name: name.into(),
            parent,
            mesh: NONE,
            rest: Trs::new(Vec3::from_array(offset), Quat::IDENTITY, Vec3::ONE),
        };
        let skeleton = SkeletonAsset {
            nodes: vec![
                node("Hips", NONE, [0.0, 0.95, 0.0]),
                node("Spine1", 0, [0.0, 0.12, 0.0]),
                node("Spine2", 1, [0.0, 0.12, 0.0]),
                node("Chest", 2, [0.0, 0.12, 0.0]),
                node("Neck1", 3, [0.0, 0.1, 0.0]),
                node("Head", 4, [0.0, 0.12, 0.0]),
                node("LeftLeg", 0, [0.1, -0.05, 0.0]),
                node("LeftShin", 6, [0.0, -0.42, 0.0]),
                node("LeftFoot", 7, [0.0, -0.42, 0.0]),
                node("LeftToeBase", 8, [0.0, -0.03, 0.12]),
                node("RightLeg", 0, [-0.1, -0.05, 0.0]),
                node("RightShin", 10, [0.0, -0.42, 0.0]),
                node("RightFoot", 11, [0.0, -0.42, 0.0]),
                node("RightToeBase", 12, [0.0, -0.03, 0.12]),
            ],
            ..Default::default()
        };
        skeleton.validate().unwrap();
        let map = HumanoidMap::with_scheme(&skeleton, "soma").unwrap();
        (skeleton, map)
    }

    /// Twenty samples of a hop: the hips rise 8 cm over flight (takeoff 4,
    /// apex 9, landing 14) and then sink `compression` per sample.
    fn hop(skeleton: &SkeletonAsset, compression: f32) -> (Vec<Vec<Trs>>, Plan) {
        let base: Vec<Vec<Trs>> = (0..20)
            .map(|t| {
                let mut pose = skeleton.rest_pose();
                let arc = if (4..=14).contains(&t) {
                    let u = (t as f32 - 9.0) / 5.0;
                    0.08 * (1.0 - u * u)
                } else if t > 14 {
                    -compression * (t - 14) as f32
                } else {
                    0.0
                };
                pose[0].translation.y += arc;
                pose
            })
            .collect();
        let plan = Plan {
            group: vec!["synthetic".into(), "hop".into()],
            clip: "hop".into(),
            takeoff: 4,
            apex: 9,
            landing: 14,
            recovery: 18,
            up: [0.0, 1.0, 0.0],
            contacts: Vec::new(),
            root: Vec::new(),
            elevation: Vec::new(),
        };
        (base, plan)
    }

    fn with_roots(
        skeleton: &SkeletonAsset,
        map: &HumanoidMap,
        base: &[Vec<Trs>],
        mut plan: Plan,
    ) -> Plan {
        plan.root = roots(skeleton, map, base)
            .into_iter()
            .map(|r| Root {
                position: [r.position[0], 0.0, r.position[2]],
                yaw: r.yaw,
            })
            .collect();
        plan
    }

    #[test]
    fn landing_on_a_straight_leg_keeps_every_joint_step_under_the_raw_bound() {
        let (skeleton, map) = rig();
        let (base, plan) = hop(&skeleton, 0.02);
        // The left foot plants where the generated foot lands, at full
        // reach, then the generated pelvis sinks 2 cm a sample: solved one
        // sample at a time the knee leaves the hip-foot line by
        // sqrt(0.42^2 - 0.41^2) = 9.1 cm in one sample, against a raw
        // bound of 2.9 cm (the last flight step).
        let anchor = skeleton.model_space(&base[14])[8].translation;
        let mut plan = with_roots(&skeleton, &map, &base, plan);
        plan.contacts.push(Contact {
            foot: 24,
            begin: 14,
            end: 20,
            anchor: anchor.to_array(),
        });
        let mut result = evaluate(&skeleton, &map, &base, &plan, "synthetic", true, None);
        let review = result.as_object_mut().unwrap().remove("review").unwrap();
        assert_eq!(review["corrected"].as_array().unwrap().len(), base.len());
        assert_eq!(review["raw"][14][8], json!(anchor.to_array()));
        assert_eq!(
            result,
            evaluate(&skeleton, &map, &base, &plan, "synthetic", false, None)
        );
        let bound = result["landing_step_bound_m"].as_f64().unwrap();
        assert!((bound - 0.0288).abs() < 0.0005, "{bound}");
        let step = result["corrected"]["landing_max_step_m"].as_f64().unwrap();
        assert!(
            step <= bound + 0.00001,
            "{step} > {bound} at {} s{}",
            result["corrected"]["landing_max_step_joint"],
            result["corrected"]["landing_max_step_sample"]
        );
        assert!(result["corrected"]["contact_max_m"].as_f64().unwrap() < 0.0001);
        assert!(result["corrected"]["flight_max_degrees"].as_f64().unwrap() <= 5.0);
        assert_eq!(result["conflicts"], 0, "{}", result["conflict_report"]);
        // The slack that made it possible is the pelvis, inside its box
        // (a quarter of the 0.84 m leg), not a relaxed contact or bound.
        let pelvis = result["pelvis_max_offset_m"].as_f64().unwrap();
        assert!(pelvis > 0.001 && pelvis <= 0.21 + 0.000001, "{pelvis}");
    }

    #[test]
    fn unreachable_contact_reports_its_triangle_bound_and_lands_at_it() {
        let (skeleton, map) = rig();
        let (base, plan) = hop(&skeleton, 0.0);
        // An anchor 1 m ahead of the hip at the foot's height: no pelvis
        // height in the box reaches it. The deficit is least with the
        // pelvis at the bottom of its box (hip 0.69 - 0.06 = 0.63 m above
        // the anchor): sqrt(1 + 0.63^2) - 0.84.
        let foot = skeleton.model_space(&base[14])[8].translation;
        let anchor = foot + Vec3::Z;
        let mut plan = with_roots(&skeleton, &map, &base, plan);
        plan.contacts.push(Contact {
            foot: 24,
            begin: 14,
            end: 15,
            anchor: anchor.to_array(),
        });
        let result = evaluate(&skeleton, &map, &base, &plan, "synthetic", false, None);
        let expected = (1.0_f64 + 0.63 * 0.63).sqrt() - 0.84;
        let report = result["conflict_report"].as_array().unwrap();
        let reach: Vec<_> = report
            .iter()
            .filter(|c| c["kind"] == "contact reach")
            .collect();
        assert_eq!(reach.len(), 1, "{report:?}");
        assert_eq!(reach[0]["sample"], 14);
        let deficit = reach[0]["deficit_m"].as_f64().unwrap();
        assert!(
            (deficit - expected).abs() < 0.001,
            "{deficit} != {expected}"
        );
        let error = result["corrected"]["contact_max_m"].as_f64().unwrap();
        assert!((error - expected).abs() < 0.002, "{error} != {expected}");
        // The pelvis went to the bottom of its box for it, not beyond.
        let pelvis = result["pelvis_max_offset_m"].as_f64().unwrap();
        assert!((pelvis - 0.21).abs() < 0.002, "{pelvis}");
    }
}
