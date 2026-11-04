//! CPU-only experiment, not a runtime path. Inputs are source-locked plans and
//! recorded poses; outputs are measurements. Never writes a world or an asset.
use og_anim::{HumanoidMap, JointLimit, NONE, SkeletonAsset, Trs};
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
    heading: f32,
    contacts: Vec<Contact>,
    #[serde(default)]
    root: Vec<Root>,
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
    } else {
        smooth((plan.recovery - sample) as f32 / (plan.recovery - plan.landing) as f32)
    }
}

fn lean(a: Vec3, b: Vec3, up: Vec3) -> f32 {
    let d = (b - a).normalize();
    og_core::math::atan2(d.cross(up).length(), d.dot(up)).to_degrees()
}

// Port of Daniel Holden's Spring-It-On (MIT), common.h and inertialization.c,
// revision 4d97b497a78e40c7b47d49e8a9da27e4aa5616d6.
// https://github.com/orangeduck/Spring-It-On
// License: third_party/spring-it-on/LICENSE. Evaluate the initial offset at
// absolute sample age, using og-core's deterministic exponential rather than
// integrating state with render dt (or the upstream rational approximation).
fn decay(x: Vec3, v: Vec3, half_life: f32, age: f32) -> Vec3 {
    let y = 2.0 * std::f32::consts::LN_2 / (half_life + 0.00001);
    (x + (v + x * y) * age) * og_core::math::exp(-y * age)
}

fn rotation_vector(mut q: Quat) -> Vec3 {
    if q.w < 0.0 {
        q = -q;
    }
    let v = Vec3::new(q.x, q.y, q.z);
    let length = v.length();
    if length == 0.0 {
        Vec3::ZERO
    } else {
        v * (2.0 * og_core::math::atan2(length, q.w) / length)
    }
}

fn rotation(v: Vec3) -> Quat {
    let length = v.length();
    if length == 0.0 {
        Quat::IDENTITY
    } else {
        og_core::math::quat_from_axis_angle(v / length, length)
    }
}

fn inertialize(raw: &[Vec<Trs>], base: &[Vec<Trs>], plan: &Plan, through: usize) -> Vec<Vec<Trs>> {
    let mut output: Vec<Vec<Trs>> = Vec::new();
    let mut offsets = vec![(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO); raw[0].len()];
    let mut transition = 0;
    let half_life = (plan.recovery - plan.landing) as f32 / 60.0;
    for t in 0..=through {
        let mut frame = raw[t].clone();
        let boundary = t == plan.takeoff
            || t == plan.landing
            || t == plan.recovery
            || plan.contacts.iter().any(|c| t == c.begin || t == c.end);
        if t > 0 && boundary {
            transition = t;
            for j in 0..frame.len() {
                // Smooth the correction, not the underlying recorded motion.
                // Otherwise a change of correction also damps real root travel.
                let delta = |p: Trs, time: usize| Trs {
                    translation: p.translation - base[time][j].translation,
                    rotation: p.rotation * base[time][j].rotation.conjugate(),
                    scale: Vec3::ONE,
                };
                let before = delta(output[t - 1][j], t - 1);
                let earlier = delta(output[t.saturating_sub(2)][j], t.saturating_sub(2));
                let next_time = (t + 1).min(raw.len() - 1);
                let next = delta(raw[next_time][j], next_time);
                let current = delta(frame[j], t);
                let src_v = rotation_vector(before.rotation * earlier.rotation.conjugate()) * 30.0;
                let dst_v = rotation_vector(next.rotation * current.rotation.conjugate()) * 30.0;
                let src_pv = (before.translation - earlier.translation) * 30.0;
                let dst_pv = (next.translation - current.translation) * 30.0;
                offsets[j] = (
                    rotation_vector(
                        rotation(src_v / 30.0) * before.rotation * current.rotation.conjugate(),
                    ),
                    src_v - dst_v,
                    before.translation + src_pv / 30.0 - current.translation,
                    src_pv - dst_pv,
                );
            }
        }
        let age = (t - transition) as f32 / 30.0;
        for (j, p) in frame.iter_mut().enumerate() {
            let (r, v, x, pv) = offsets[j];
            p.rotation = og_anim::unit(rotation(decay(r, v, half_life, age)) * p.rotation);
            p.translation += decay(x, pv, half_life, age);
        }
        output.push(frame);
    }
    output
}

fn solve(
    skeleton: &SkeletonAsset,
    base: &[Trs],
    map: &HumanoidMap,
    plan: &Plan,
    sample: usize,
    scale: f32,
) -> (Vec<Trs>, usize) {
    let mut pose = base.to_vec();
    let hips = map.node("hips").unwrap();
    let chest = map.node("chest").unwrap();
    let head = map.node("head").unwrap();
    let up = Vec3::from_array(plan.up);
    let mut conflicts = 0;
    // Fixed-order alternating chest/head constraints. Existing FABRIK keeps
    // translations/rest lengths; its joint-limit path handles constrained reach.
    for _ in 0..4 {
        for tip in [chest, head] {
            let mut chain = vec![tip];
            while *chain.last().unwrap() != hips {
                chain.push(skeleton.nodes[*chain.last().unwrap() as usize].parent);
            }
            chain.reverse();
            let world = skeleton.model_space(&pose);
            let origin = world[hips as usize].translation;
            let length = (world[tip as usize].translation - origin).length();
            let limits: Vec<_> = chain
                .iter()
                .map(|&joint| {
                    Some(JointLimit {
                        joint,
                        swing: if joint == hips { 1.05 } else { 0.79 },
                        twist_min: -1.57,
                        twist_max: 1.57,
                        ..Default::default()
                    })
                })
                .collect();
            let result = og_anim::fabrik(
                skeleton,
                &mut pose,
                &chain,
                origin + up * length,
                12,
                0.0001,
                1.0,
                &limits,
            );
            conflicts += usize::from(!result.reachable);
        }
    }
    // Blend once, after convergence. Blending every alternating projection
    // made the effective recovery curve depend on the number of iterations.
    for (p, original) in pose.iter_mut().zip(base) {
        p.rotation = og_anim::nlerp(original.rotation, p.rotation, weight(plan, sample));
    }
    conflicts += plant(skeleton, &mut pose, base, map, plan, sample, scale);
    (pose, conflicts)
}

fn plant(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    base: &[Trs],
    map: &HumanoidMap,
    plan: &Plan,
    sample: usize,
    scale: f32,
) -> usize {
    let hips = map.node("hips").unwrap();
    let mut conflicts = 0;
    let world = skeleton.model_space(pose);
    let base_world = skeleton.model_space(base);
    let mut targets = Vec::new();
    for foot in [24, 28] {
        let side = if foot == 24 { "left" } else { "right" };
        let chain = ["UpperLeg", "LowerLeg", "Foot"]
            .map(|bone| map.node(&format!("{side}{bone}")).unwrap());
        let generated = base_world[chain[2] as usize];
        let contact = plan
            .contacts
            .iter()
            .filter(|c| c.foot == foot)
            .filter(|c| {
                let approach = if c.begin >= plan.landing {
                    plan.apex
                } else {
                    0
                };
                let release = if c.end <= plan.takeoff {
                    plan.takeoff
                } else {
                    plan.recovery
                };
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
            targets.push((chain, generated.translation, generated.rotation));
            continue;
        };
        // Approach the authored landing anchor during flight, before the hard
        // contact begins. Contact itself never fades or follows generated flags.
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
        let w = if sample < contact.begin {
            smooth((sample - approach) as f32 / (contact.begin - approach) as f32)
        } else if sample < contact.end {
            1.0
        } else {
            smooth((release - sample) as f32 / (release - contact.end) as f32)
        };
        targets.push((
            chain,
            generated
                .translation
                .lerp(Vec3::from_array(contact.anchor) * scale, w),
            generated.rotation,
        ));
    }
    // Pelvis height is the slack variable. Each leg gives an interval of
    // feasible heights; intersect them before solving either knee. The
    // authoritative root is not an input or an output of this operator.
    let leg = ["leftUpperLeg", "leftLowerLeg", "leftFoot"].map(|b| map.node(b).unwrap() as usize);
    let leg_length = (world[leg[1]].translation - world[leg[0]].translation).length()
        + (world[leg[2]].translation - world[leg[1]].translation).length();
    let existing = world[hips as usize].translation.y - base_world[hips as usize].translation.y;
    let (mut low, mut high) = (-0.25 * leg_length - existing, 0.10 * leg_length - existing);
    for &(chain, target, _) in &targets {
        let [a, b, c] = chain.map(|n| world[n as usize].translation);
        let reach = (b - a).length() + (c - b).length() - 0.00001;
        let d = a - target;
        let square = reach * reach - d.x * d.x - d.z * d.z;
        if square < 0.0 {
            conflicts += 1;
            continue;
        }
        let extent = square.sqrt();
        low = low.max(-extent - d.y);
        high = high.min(extent - d.y);
    }
    let offset = if low <= high {
        0.0_f32.clamp(low, high)
    } else {
        conflicts += 1;
        ((low + high) * 0.5).clamp(-0.25 * leg_length - existing, 0.10 * leg_length - existing)
    };
    let parent = skeleton.nodes[hips as usize].parent;
    let delta = if parent == NONE {
        Vec3::Y * offset
    } else {
        world[parent as usize].rotation.conjugate() * (Vec3::Y * offset)
            / world[parent as usize].scale
    };
    pose[hips as usize].translation += delta;
    for (chain, target, orientation) in targets {
        let pole = skeleton.model_space(pose)[chain[0] as usize].translation
            + Vec3::new(
                og_core::math::sin(plan.heading),
                0.0,
                og_core::math::cos(plan.heading),
            ) * leg_length;
        let result = og_anim::two_bone(skeleton, pose, &chain, target, Some(pole), 1.0, &[]);
        conflicts += usize::from(!result.reachable);
        let world = skeleton.model_space(pose);
        pose[chain[2] as usize].rotation =
            world[chain[1] as usize].rotation.conjugate() * orientation;
    }
    assert!(
        offset + existing >= -0.25 * leg_length - 0.000001
            && offset + existing <= 0.10 * leg_length + 0.000001
    );
    for (j, (a, b)) in pose.iter().zip(base).enumerate() {
        if j != hips as usize {
            assert_eq!(a.translation, b.translation);
        }
        assert_eq!(a.scale, b.scale);
    }
    conflicts
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
) -> serde_json::Value {
    // Keep the original raw-root baseline and its continuity bound. Neither
    // re-rooting nor a later solve may relax that recorded acceptance bound.
    let before = measure(skeleton, map, base, plan, 1.0);
    let placed = aligned(skeleton, map, base, plan);
    let aligned_base = measure(skeleton, map, &placed, plan, 1.0);
    let base = &placed;
    let mut conflicts = 0;
    let raw: Vec<_> = base
        .iter()
        .enumerate()
        .map(|(t, p)| {
            let (p, n) = solve(skeleton, p, map, plan, t, 1.0);
            conflicts += n;
            p
        })
        .collect();
    let mut corrected = inertialize(&raw, base, plan, raw.len() - 1);
    // Upright posture has priority over the soft inertial residual. Keeping
    // this constrained chain on its C1 target also preserves torso joint limits.
    let mut torso = vec![map.node("head").unwrap()];
    while *torso.last().unwrap() != map.node("hips").unwrap() {
        torso.push(skeleton.nodes[*torso.last().unwrap() as usize].parent);
    }
    let project = |t: usize, p: &mut [Trs]| {
        for &j in &torso {
            p[j as usize].rotation = raw[t][j as usize].rotation;
        }
        plant(skeleton, p, &base[t], map, plan, t, 1.0)
    };
    for (t, p) in corrected.iter_mut().enumerate() {
        // Inertialization cannot override a hard planned foot contact.
        conflicts += project(t, p);
    }
    // Random access/seek and sparse draws are evaluated independently;
    // equality is exact, not a float tolerance.
    for t in (0..base.len()).rev().chain((0..base.len()).step_by(7)) {
        let mut sought = inertialize(&raw, base, plan, t).pop().unwrap();
        project(t, &mut sought);
        assert_eq!(corrected[t], sought);
    }
    let after = measure(skeleton, map, &corrected, plan, 1.0);
    let hips = map.node("hips").unwrap() as usize;
    let pelvis_offset = corrected
        .iter()
        .zip(base)
        .map(|(a, b)| {
            (skeleton.model_space(a)[hips].translation - skeleton.model_space(b)[hips].translation)
                .length()
        })
        .fold(0.0_f32, f32::max);
    json!({"group": plan.group, "clip": plan.clip, "rig": rig,
        "frames": base.len(), "conflicts": conflicts,
        "base": before, "aligned_base": aligned_base, "corrected": after, "pelvis_max_offset_m": pelvis_offset,
        "landing_step_bound_m": before["landing_max_step_m"],
        "landing_step_pass": after["landing_max_step_m"].as_f64().unwrap() <= before["landing_max_step_m"].as_f64().unwrap()+0.00001,
        "seek_replay_sparse_equal": true})
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
        if hash(drawn) != *expected && replay_mismatch.is_none() {
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
        let mut result = evaluate(&skeleton, &map, &base, &recording.plan, "soma");
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
        let mut result = evaluate(&target, &target_map, &retargeted, &plan, "meshy-traveller");
        result["model_root_drift_soma"] = model_drift;
        results.push(result);
    }
    println!(
        "{}",
        json!({"plan_sha256": input.plan_sha256, "results": results,
        "root_provenance":"Ground-projected source hips [x,0,z] and source yaw stand in for controller root; generated pelvis Y unchanged. Horizontal/yaw correction only.",
        "original_library_training_jump_windows":0,
        "world_replay": hash_replay(),
        "limitations": ["Prototype palette not integrated", "Gesture priority and leg limits pending",
            "Controller-owned vertical arc treatment remains open before motion.jump IDL",
            "C1 correction does not imply bounded discrete joint steps"]})
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
}
