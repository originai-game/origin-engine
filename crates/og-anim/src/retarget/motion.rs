//! Clip mirroring, foot plants and humanoid motion retargeting.

use super::*;

/// The node name on the other side: the side tokens the naming schemes
/// read swapped (a `Left`/`Right` prefix after any namespace, as Mixamo,
/// Unity and plain names write it; VRM's `_L_`/`_R_`; a `.L`, `_R`, `.left`,
/// `_l` suffix, as Blender, Rigify and Unreal write it), keeping the case;
/// `None` for a name with no side.
pub fn mirror_name(name: &str) -> Option<String> {
    let cut = name.rfind([':', '|']).map_or(0, |i| i + 1);
    let (ns, local) = name.split_at(cut);
    for (a, b) in [("Left", "Right"), ("left", "right"), ("LEFT", "RIGHT")] {
        for (from, to) in [(a, b), (b, a)] {
            if let Some(rest) = local.strip_prefix(from) {
                return Some(format!("{ns}{to}{rest}"));
            }
        }
    }
    for (from, to) in [
        ("_L_", "_R_"),
        ("_R_", "_L_"),
        ("_l_", "_r_"),
        ("_r_", "_l_"),
    ] {
        if let Some(i) = local.find(from) {
            return Some(format!(
                "{ns}{}{to}{}",
                &local[..i],
                &local[i + from.len()..]
            ));
        }
    }
    for sep in ['.', '_', '-', ' '] {
        for (a, b) in [
            ("Left", "Right"),
            ("left", "right"),
            ("LEFT", "RIGHT"),
            ("L", "R"),
            ("l", "r"),
        ] {
            for (from, to) in [(a, b), (b, a)] {
                if let Some(rest) = local.strip_suffix(from)
                    && rest.ends_with(sep)
                {
                    return Some(format!("{ns}{rest}{to}"));
                }
            }
        }
    }
    None
}

/// Each node's counterpart on the other side ([`mirror_name`]): itself for
/// a node without a side, [`NONE`] for a sided node whose counterpart the
/// skeleton lacks (or whose counterpart does not name it back).
pub fn mirror_pairs(skeleton: &SkeletonAsset) -> Vec<u32> {
    (0..skeleton.nodes.len())
        .map(|i| match mirror_name(&skeleton.nodes[i].name) {
            None => i as u32,
            Some(other) => skeleton
                .node(&other)
                .filter(|&k| {
                    mirror_name(&skeleton.nodes[k as usize].name).as_deref()
                        == Some(skeleton.nodes[i].name.as_str())
                })
                .unwrap_or(NONE),
        })
        .collect()
}

/// `v` reflected in the model's x = 0 plane.
fn reflect(v: Vec3) -> Vec3 {
    Vec3::new(-v.x, v.y, v.z)
}

/// A rotation reflected in the model's x = 0 plane (`M q M`).
fn reflect_rotation(q: Quat) -> Quat {
    Quat::from_xyzw(q.x, -q.y, -q.z, q.w)
}

/// `clip` (made for `skeleton`) played on the other side: every frame
/// reflected in the model's x = 0 plane, each node taking its
/// counterpart's motion ([`mirror_pairs`]). Rotations move in model space
/// (a node's model rotation is its counterpart's change from rest,
/// reflected, applied to its own rest), so left and right bone frames need
/// not mirror each other; translations are the counterpart's travel from
/// rest, reflected (root motion's x and yaw flip); scales and blendshape
/// weights are copied, visibility channels move to the counterpart.
/// Assumes a symmetric rest pose. A node gets a channel
/// when it or its counterpart is animated; a sided node without a
/// counterpart gets none (it keeps its rest pose). Mirroring twice gives
/// the clip back. Same name, duration, rate and markers.
pub fn mirror(skeleton: &SkeletonAsset, clip: &ClipAsset) -> Result<ClipAsset> {
    clip.validate()?;
    let pairs = mirror_pairs(skeleton);
    let binding = bind(skeleton, clip);
    let n = skeleton.nodes.len();
    let animated = |node: u32, path: ChannelPath| {
        node != NONE
            && clip
                .channels
                .iter()
                .zip(&binding)
                .any(|(c, &b)| b == node && c.path == path)
    };
    let wants = |path: ChannelPath| -> Vec<bool> {
        (0..n)
            .map(|i| pairs[i] != NONE && (animated(i as u32, path) || animated(pairs[i], path)))
            .collect()
    };
    let (rot, pos, scl) = (
        wants(ChannelPath::Rotation),
        wants(ChannelPath::Translation),
        wants(ChannelPath::Scale),
    );
    let rest_pose = skeleton.rest_pose();
    let rest = model(skeleton, &rest_pose);
    let mut samples: Vec<[Vec<f32>; 3]> = vec![Default::default(); n];
    for i in 0..clip.frames {
        let t = (i as f32 / clip.rate).min(clip.duration);
        let mut src = Pose::rest(skeleton);
        sample(skeleton, clip, &binding, t, NONE, &mut src);
        let ms = model(skeleton, &src.nodes);
        let mut pose = rest_pose.clone();
        let mut mt: Vec<Trs> = Vec::with_capacity(n);
        for j in 0..n {
            let parent = parent_model(skeleton, &mt, j);
            let k = pairs[j] as usize;
            if rot[j] {
                let change = ms[k].rotation * rest[k].rotation.conjugate();
                let want = unit(reflect_rotation(change) * rest[j].rotation);
                pose[j].rotation = unit(parent.rotation.conjugate() * want);
                samples[j][1].extend_from_slice(&pose[j].rotation.to_array());
            }
            if pos[j] {
                let at = rest[j].translation + reflect(ms[k].translation - rest[k].translation);
                pose[j].translation = parent.inverse().transform_point(at);
                samples[j][0].extend_from_slice(&pose[j].translation.to_array());
            }
            if scl[j] {
                pose[j].scale = src.nodes[k].scale;
                samples[j][2].extend_from_slice(&pose[j].scale.to_array());
            }
            mt.push(parent.mul(&pose[j]));
        }
    }
    let mut channels = Vec::new();
    for (j, s) in samples.into_iter().enumerate() {
        for (path, lanes, data) in [
            (ChannelPath::Translation, 3, &s[0]),
            (ChannelPath::Rotation, 4, &s[1]),
            (ChannelPath::Scale, 3, &s[2]),
        ] {
            if !data.is_empty() {
                channels.push(Channel {
                    node: j as u32,
                    node_name: skeleton.nodes[j].name.clone(),
                    path,
                    step: false,
                    lanes,
                    samples: data.clone().into(),
                });
            }
        }
    }
    for (c, &b) in clip.channels.iter().zip(&binding) {
        if c.path == ChannelPath::Weights && b != NONE {
            channels.push(Channel {
                node: b,
                node_name: skeleton.nodes[b as usize].name.clone(),
                ..c.clone()
            });
        }
    }
    channels.extend(crate::visibility::mirrored(
        skeleton, clip, &binding, &pairs,
    ));
    let out = ClipAsset {
        channels,
        ..clip.clone()
    };
    out.validate()?;
    Ok(out)
}

/// How far above its lowest a foot may be and still be planted, in the
/// skeleton's model units: 5 mm.
pub const PLANT_HEIGHT: f32 = 0.005;

/// Where a foot meets the ground over a clip: the one notion of "planted"
/// the engine has (retargeting's contact checks, `og.FootLock`). The foot
/// node's model-space height (the clip sampled over the rest pose, root
/// motion not removed) at each of the clip's frames; a frame is planted when
/// the foot is within [`PLANT_HEIGHT`] of its lowest over the clip.
#[derive(Clone, Debug, PartialEq)]
pub struct FootPlants {
    /// The foot's model-space height at each frame.
    pub heights: Vec<f32>,
    /// The lowest of them (the clip's ground for this foot).
    pub lowest: f32,
}

impl FootPlants {
    /// The plants of node `foot` of `skeleton` over `clip` (made for it).
    pub fn new(skeleton: &SkeletonAsset, clip: &ClipAsset, foot: u32) -> FootPlants {
        let binding = bind(skeleton, clip);
        let heights: Vec<f32> = (0..clip.frames)
            .map(|i| {
                let t = (i as f32 / clip.rate).min(clip.duration);
                let mut pose = Pose::rest(skeleton);
                sample(skeleton, clip, &binding, t, NONE, &mut pose);
                model(skeleton, &pose.nodes)[foot as usize].translation.y
            })
            .collect();
        let lowest = heights.iter().copied().fold(f32::MAX, f32::min);
        FootPlants { heights, lowest }
    }

    /// Whether the foot is planted at frame `frame`.
    pub fn planted(&self, frame: usize) -> bool {
        self.heights
            .get(frame)
            .is_some_and(|&h| h <= self.lowest + PLANT_HEIGHT)
    }

    /// Whether the foot is planted at clip time `t` of `clip` (the clip the
    /// plants were made from): its height interpolated between the frames
    /// around `t`, as the frames are.
    pub fn planted_at(&self, clip: &ClipAsset, t: f32) -> bool {
        let p = clip.frame_pos(t);
        let (Some(&a), Some(&b)) = (self.heights.get(p.i0), self.heights.get(p.i1)) else {
            return false;
        };
        a + (b - a) * p.frac <= self.lowest + PLANT_HEIGHT
    }
}

/// How [`retarget`] treats the legs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetargetOptions {
    /// Solve each leg so the foot lands where the source's foot does,
    /// scaled by the leg ratio (planted feet stay on the target's ground).
    pub foot_ik: bool,
}

impl Default for RetargetOptions {
    fn default() -> Self {
        RetargetOptions { foot_ik: true }
    }
}

/// What [`retarget`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct Retargeted {
    /// The clip for the target skeleton (same name, duration, rate and markers).
    pub clip: ClipAsset,
    /// The leg-height ratio target / source the hips' travel was scaled by.
    pub scale: f32,
    /// Standard bones mapped on both rigs.
    pub bones: u32,
    /// Legs solved for the feet (0, 1 or 2).
    pub legs: u32,
    /// Twist bones of the target driven from their limb segments.
    pub twists: u32,
}

/// The height of the foot of `leg` (0 left, 1 right) above the ground the
/// character stands on: the rig's `footHeight`, else the rest pose's.
fn foot_y(rest: &[Trs], map: &HumanoidMap, leg: usize) -> f32 {
    map.foot_height
        .map_or(rest[map.nodes[LEG[leg] + 2] as usize].translation.y, |h| {
            h[leg]
        })
}

/// A twist bone of the target (a node no standard bone maps whose name
/// says `twist`, or a Rigify deform segment `….001`) on the limb segment
/// from standard bone `a` to the next one, `c`.
#[derive(Clone, Debug)]
pub(crate) struct TwistBone {
    pub(crate) node: usize,
    /// The segment's bone and the next bone's node; `a`'s parent node.
    pub(crate) a: usize,
    pub(crate) c: usize,
    parent: Option<usize>,
    /// The segment's direction in `a`'s frame (towards `c` at rest).
    axis: Vec3,
    /// How far along the segment the twist bone sits, 0 at `a` to 1 at `c`.
    at: f32,
}

/// Whether a node name is a twist bone's.
fn twist_name(name: &str) -> bool {
    let lower = local_name(name).to_lowercase();
    lower.contains("twist")
        || (lower.starts_with("def-")
            && lower.len() > 4
            && lower[lower.len() - 4..].starts_with('.')
            && lower[lower.len() - 3..].bytes().all(|b| b.is_ascii_digit()))
}

/// The twist of rotation `q` about unit `axis` (swing-twist), in radians.
fn twist_angle(q: Quat, axis: Vec3) -> f32 {
    let p = Vec3::new(q.x, q.y, q.z).dot(axis);
    let (p, w) = if q.w < 0.0 { (-p, -q.w) } else { (p, q.w) };
    2.0 * og_core::math::atan2(p, w)
}

/// The twist bones of `skeleton` (mapped by `map`), on the upper and
/// lower arm and leg segments whose next bone is mapped.
pub(crate) fn twist_bones(
    skeleton: &SkeletonAsset,
    map: &HumanoidMap,
    rest: &[Trs],
) -> Vec<TwistBone> {
    let mapped = |n: usize| map.nodes.iter().position(|&m| m as usize == n);
    let mut out = Vec::new();
    for (n, node) in skeleton.nodes.iter().enumerate() {
        if mapped(n).is_some() || !twist_name(&node.name) {
            continue;
        }
        // The nearest mapped ancestor is the segment.
        let mut p = node.parent;
        let a_bone = loop {
            if p == NONE {
                break None;
            }
            if let Some(b) = mapped(p as usize) {
                break Some(b);
            }
            p = skeleton.nodes[p as usize].parent;
        };
        let Some(ab) = a_bone else { continue };
        let segment = [
            ARM[0] + 1,
            ARM[0] + 2,
            ARM[1] + 1,
            ARM[1] + 2,
            LEG[0],
            LEG[0] + 1,
        ]
        .into_iter()
        .chain([LEG[1], LEG[1] + 1])
        .any(|b| b == ab);
        if !segment || map.nodes[ab + 1] == NONE {
            continue;
        }
        let (a, c) = (map.nodes[ab] as usize, map.nodes[ab + 1] as usize);
        let along = rest[c].translation - rest[a].translation;
        let len2 = along.length_squared();
        if len2 < 1e-12 {
            continue;
        }
        let local = rest[a].inverse().transform_point(rest[c].translation);
        let axis = local / local.length();
        let at = ((rest[n].translation - rest[a].translation).dot(along) / len2).clamp(0.0, 1.0);
        let parent = match skeleton.nodes[a].parent {
            NONE => None,
            q => Some(q as usize),
        };
        out.push(TwistBone {
            node: n,
            a,
            c,
            parent,
            axis,
            at,
        });
    }
    out
}

/// Turns each of `twists` in `pose` (whose model space is `fk`; `rest` is
/// the rest pose's) to a linear share of its segment's twist, from none
/// relative to the segment's parent at its head to the next bone's at its
/// end. Every node `keep` names keeps its model rotation from `fk`; the
/// others keep their local rotation.
pub(crate) fn share_twist(
    skeleton: &SkeletonAsset,
    twists: &[&TwistBone],
    rest: &[Trs],
    fk: &[Trs],
    pose: &mut [Trs],
    keep: impl Fn(usize) -> bool,
) {
    let mut m2: Vec<Trs> = Vec::with_capacity(pose.len());
    for n in 0..pose.len() {
        let p = parent_model(skeleton, &m2, n);
        if let Some(tw) = twists.iter().find(|t| t.node == n) {
            let rel = |of: &[Trs], x: usize, y: Option<usize>| -> Quat {
                let base = y.map_or(Quat::IDENTITY, |y| of[y].rotation);
                base.conjugate() * of[x].rotation
            };
            let tau_a = twist_angle(
                rel(rest, tw.a, tw.parent).conjugate() * rel(fk, tw.a, tw.parent),
                tw.axis,
            );
            let tau_c = twist_angle(
                rel(fk, tw.c, Some(tw.a)) * rel(rest, tw.c, Some(tw.a)).conjugate(),
                tw.axis,
            );
            let theta = tw.at * tau_c - (1.0 - tw.at) * tau_a;
            let q = og_core::math::quat_from_axis_angle(tw.axis, theta);
            let want = unit(m2[tw.a].rotation * q * rel(rest, n, Some(tw.a)));
            pose[n].rotation = unit(p.rotation.conjugate() * want);
        } else if keep(n) {
            pose[n].rotation = unit(p.rotation.conjugate() * fk[n].rotation);
        }
        m2.push(p.mul(&pose[n]));
    }
}

/// The height of the hips above the feet at rest (model space), or of the
/// hips alone when no foot is mapped.
fn leg_height(rest: &[Trs], map: &HumanoidMap) -> f32 {
    let hips = rest[map.nodes[HIPS] as usize].translation.y;
    let feet: Vec<f32> = [0, 1]
        .into_iter()
        .filter(|&l| map.nodes[LEG[l] + 2] != NONE)
        .map(|l| foot_y(rest, map, l))
        .collect();
    if feet.is_empty() {
        hips
    } else {
        hips - feet.iter().sum::<f32>() / feet.len() as f32
    }
}

/// Model space of a pose (the skeleton's own composition).
fn model(skeleton: &SkeletonAsset, pose: &[Trs]) -> Vec<Trs> {
    skeleton.model_space(pose)
}

/// The parent's model transform of node `n` (identity at a root).
fn parent_model(skeleton: &SkeletonAsset, m: &[Trs], n: usize) -> Trs {
    match skeleton.nodes[n].parent {
        NONE => Trs::IDENTITY,
        p => m[p as usize],
    }
}

/// Turns node `n` (in `pose`) so that its model rotation becomes `rotation`.
fn set_model_rotation(
    skeleton: &SkeletonAsset,
    pose: &mut [Trs],
    m: &[Trs],
    n: usize,
    rotation: Quat,
) {
    let p = parent_model(skeleton, m, n);
    pose[n].rotation = unit(p.rotation.conjugate() * rotation);
}

/// Rewrites `clip` (made for `source`) for `target`. Both maps need the
/// hips; bones mapped on one rig only are left out, and target nodes no
/// mapped bone covers keep their rest pose. A target bone gets a channel
/// when the clip animates its source bone (and both legs' bones when their
/// feet are solved), so a partial clip (a wave) stays partial. Visibility
/// channels follow their bone, else their node's name.
pub fn retarget(
    source: &SkeletonAsset,
    source_map: &HumanoidMap,
    target: &SkeletonAsset,
    target_map: &HumanoidMap,
    clip: &ClipAsset,
    options: &RetargetOptions,
) -> Result<Retargeted> {
    for (map, which, skeleton) in [
        (source_map, "source", source),
        (target_map, "target", target),
    ] {
        if map.nodes[HIPS] == NONE {
            return Err(invalid(format!(
                "the {which} skeleton has no hips the engine recognises (scheme '{}'); name them in a `type rig` .ogt (`hips \"<node>\"`)",
                map.scheme
            )));
        }
        if let Some(&n) = map
            .nodes
            .iter()
            .find(|&&n| n != NONE && n as usize >= skeleton.nodes.len())
        {
            return Err(invalid(format!(
                "the {which} map names node {n}, past the skeleton's {} nodes",
                skeleton.nodes.len()
            )));
        }
    }
    clip.validate()?;
    let binding = bind(source, clip);
    let animated = |node: u32, path: Option<ChannelPath>| {
        clip.channels
            .iter()
            .zip(&binding)
            .any(|(c, &b)| b == node && path.is_none_or(|p| c.path == p))
    };
    let both = |b: usize| source_map.nodes[b] != NONE && target_map.nodes[b] != NONE;

    let s_rest = model(source, &source.rest_pose());
    let t_rest_pose = target.rest_pose();
    let t_rest = model(target, &t_rest_pose);
    let (hs, ht) = (
        leg_height(&s_rest, source_map),
        leg_height(&t_rest, target_map),
    );
    let k = if hs.abs() > 1e-6 && ht.abs() > 1e-6 {
        ht / hs
    } else {
        1.0
    };

    // The rest correction of each bone: the target's rest bone direction
    // turned onto the source's; inherited by bones with no child but the
    // head, none for the hips. The head looks where the source's looks
    // (none either): inherited, a neck leaning at rest (SOMA's, 17°) would
    // pitch it by the lean.
    let mut correction = [Quat::IDENTITY; HUMAN_BONE_COUNT];
    for b in 1..HUMAN_BONE_COUNT {
        if !both(b) {
            continue;
        }
        let (sb, tb) = (source_map.nodes[b] as usize, target_map.nodes[b] as usize);
        correction[b] = match children(b).iter().find(|&&c| both(c)) {
            Some(&c) => {
                let (sc, tc) = (source_map.nodes[c] as usize, target_map.nodes[c] as usize);
                rotation_arc(
                    t_rest[tc].translation - t_rest[tb].translation,
                    s_rest[sc].translation - s_rest[sb].translation,
                )
            }
            None if b == HEAD => Quat::IDENTITY,
            None => {
                let mut p = parent_bone(b);
                while p != usize::MAX && !both(p) {
                    p = parent_bone(p);
                }
                if p == usize::MAX {
                    Quat::IDENTITY
                } else {
                    correction[p]
                }
            }
        };
    }

    // Which bones get channels, and which legs are solved.
    let mut emit = [false; HUMAN_BONE_COUNT];
    for (b, e) in emit.iter_mut().enumerate() {
        *e = both(b) && animated(source_map.nodes[b], None);
    }
    let hips_travel = animated(source_map.nodes[HIPS], Some(ChannelPath::Translation));
    let mut legs: Vec<usize> = Vec::new();
    if options.foot_ik {
        for &l in &LEG {
            let chain = [l, l + 1, l + 2];
            if chain.iter().all(|&b| both(b)) && (emit[HIPS] || chain.iter().any(|&b| emit[b])) {
                for b in chain {
                    emit[b] = true;
                }
                legs.push(l);
            }
        }
    }
    // The hips move when the clip moves them or a solved leg may lower them.
    let moves_hips = hips_travel || !legs.is_empty();
    if moves_hips {
        emit[HIPS] = true;
    }
    let th = target_map.nodes[HIPS] as usize;
    let sh = source_map.nodes[HIPS] as usize;
    // Twist bones on segments the clip moves (the segment or its next bone).
    let twists: Vec<TwistBone> = twist_bones(target, target_map, &t_rest)
        .into_iter()
        .filter(|t| {
            emit.iter().enumerate().any(|(b, &e)| {
                e && (target_map.nodes[b] as usize == t.a || target_map.nodes[b] as usize == t.c)
            })
        })
        .collect();
    let mut twist_rotations: Vec<Vec<f32>> = vec![Vec::new(); twists.len()];

    // Per frame: the target pose, recorded into the emitted channels.
    let emitted: Vec<usize> = {
        let mut v: Vec<usize> = (0..HUMAN_BONE_COUNT).filter(|&b| emit[b]).collect();
        og_core::sort::sort_by_key(&mut v, |&b| target_map.nodes[b]);
        v
    };
    let mut rotations: Vec<Vec<f32>> = vec![Vec::new(); emitted.len()];
    let mut travel: Vec<f32> = Vec::new();
    let mut by_node: Vec<Option<usize>> = vec![None; target.nodes.len()];
    for &b in &emitted {
        by_node[target_map.nodes[b] as usize] = Some(b);
    }
    for i in 0..clip.frames {
        let t = (i as f32 / clip.rate).min(clip.duration);
        let mut ps = Pose::rest(source);
        sample(source, clip, &binding, t, NONE, &mut ps);
        let ms = model(source, &ps.nodes);
        let desired = |b: usize| -> Quat {
            let s = source_map.nodes[b] as usize;
            let tb = target_map.nodes[b] as usize;
            unit(
                ms[s].rotation
                    * s_rest[s].rotation.conjugate()
                    * correction[b]
                    * t_rest[tb].rotation,
            )
        };
        let hips_at = t_rest[th].translation + (ms[sh].translation - s_rest[sh].translation) * k;
        // Forward kinematics in node order: each emitted bone at its
        // desired model rotation, the hips at their scaled travel.
        let mut pt = t_rest_pose.clone();
        let mut mt: Vec<Trs> = Vec::with_capacity(pt.len());
        for n in 0..pt.len() {
            let p = parent_model(target, &mt, n);
            if let Some(b) = by_node[n] {
                let want = match hand_of(b) {
                    // A finger turns from its hand as the source's does from
                    // its own, so a hand pose holds on an arm the clip
                    // leaves alone (A-posed under a T-posed source's clip).
                    Some(h) if both(h) => {
                        let hand = target_map.nodes[h] as usize;
                        unit(mt[hand].rotation * desired(h).conjugate() * desired(b))
                    }
                    _ => desired(b),
                };
                pt[n].rotation = unit(p.rotation.conjugate() * want);
            }
            if n == th && moves_hips {
                pt[n].translation = p.inverse().transform_point(hips_at);
            }
            mt.push(p.mul(&pt[n]));
        }
        // A leg too short for its foot's goal (a straight source leg on a
        // rig with other proportions) lowers the hips until it reaches.
        // On the ground the rigs stand on: each foot's height above it is
        // the rig's `footHeight`, else its rest pose's.
        let goal = |l: usize| {
            let side = (l == LEG[1]) as usize;
            let sf = source_map.nodes[l + 2] as usize;
            let ft = target_map.nodes[l + 2] as usize;
            let mut base = t_rest[ft].translation;
            base.y = foot_y(&t_rest, target_map, side);
            let mut from = s_rest[sf].translation;
            from.y = foot_y(&s_rest, source_map, side);
            base + (ms[sf].translation - from) * k
        };
        let mut drop = 0f32;
        for &l in &legs {
            let (up, lo, ft) = (
                target_map.nodes[l] as usize,
                target_map.nodes[l + 1] as usize,
                target_map.nodes[l + 2] as usize,
            );
            let (a, b, c) = (mt[up].translation, mt[lo].translation, mt[ft].translation);
            let reach = (b - a).length() + (c - b).length() - 1e-4;
            let v = goal(l) - a;
            if v.length() > reach {
                // The hips at a + d·y reach the goal: |v - d·y| = reach.
                let disc = v.y * v.y - (v.length_squared() - reach * reach);
                let d = if disc >= 0.0 { v.y + disc.sqrt() } else { v.y };
                drop = drop.min(d);
            }
        }
        if drop < 0.0 {
            let p = parent_model(target, &mt, th);
            pt[th].translation = p.inverse().transform_point(hips_at + Vec3::Y * drop);
            mt = model(target, &pt);
        }
        // The legs: the knee bent in its plane so the foot reaches the
        // source's foot travel, scaled; then the foot turned back to its
        // transferred rotation.
        for &l in &legs {
            let (up, lo, ft) = (
                target_map.nodes[l] as usize,
                target_map.nodes[l + 1] as usize,
                target_map.nodes[l + 2] as usize,
            );
            let goal = goal(l);
            let (a, b, c) = (mt[up].translation, mt[lo].translation, mt[ft].translation);
            let (l1, l2) = ((b - a).length(), (c - b).length());
            let to = goal - a;
            let dist = to.length();
            if l1 < 1e-6 || l2 < 1e-6 || dist < 1e-6 {
                continue;
            }
            let dir = to / dist;
            let (reach_min, reach_max) = ((l1 - l2).abs(), l1 + l2);
            let d = dist.clamp(reach_min + 1e-6, reach_max - 1e-6).max(1e-6);
            let hint = b - a;
            let mut bend = hint - dir * hint.dot(dir);
            if bend.length_squared() < 1e-12 {
                // A straight leg: bend forward (+z), as knees do.
                bend = Vec3::Z - dir * dir.z;
            }
            let bend = bend / bend.length();
            let x = (l1 * l1 - l2 * l2 + d * d) / (2.0 * d);
            let h = (l1 * l1 - x * x).max(0.0).sqrt();
            let knee = a + dir * x + bend * h;
            let tip = a + dir * d;
            let r_up = unit(rotation_arc(b - a, knee - a) * mt[up].rotation);
            set_model_rotation(target, &mut pt, &mt, up, r_up);
            let mt2 = model(target, &pt);
            let (b2, c2) = (mt2[lo].translation, mt2[ft].translation);
            let r_lo = unit(rotation_arc(c2 - b2, tip - b2) * mt2[lo].rotation);
            set_model_rotation(target, &mut pt, &mt2, lo, r_lo);
            let mt3 = model(target, &pt);
            set_model_rotation(target, &mut pt, &mt3, ft, desired(l + 2));
            mt = model(target, &pt);
        }
        // Twist bones: a linear share of the segment's twist, from none
        // relative to the segment's parent at its head to the next bone's at
        // its end; every other node keeps its model rotation.
        if !twists.is_empty() {
            let all: Vec<&TwistBone> = twists.iter().collect();
            share_twist(target, &all, &t_rest, &mt, &mut pt, |n| {
                by_node[n].is_some()
            });
            for (j, tw) in twists.iter().enumerate() {
                let r = pt[tw.node].rotation;
                twist_rotations[j].extend_from_slice(&[r.x, r.y, r.z, r.w]);
            }
        }
        for (k, &b) in emitted.iter().enumerate() {
            let r = pt[target_map.nodes[b] as usize].rotation;
            rotations[k].extend_from_slice(&[r.x, r.y, r.z, r.w]);
        }
        if moves_hips {
            travel.extend_from_slice(&pt[th].translation.to_array());
        }
    }
    let mut channels: Vec<Channel> = Vec::new();
    for (k, &b) in emitted.iter().enumerate() {
        let n = target_map.nodes[b];
        if n as usize == th && moves_hips {
            channels.push(Channel {
                node: n,
                node_name: target.nodes[n as usize].name.clone(),
                path: ChannelPath::Translation,
                step: false,
                lanes: 3,
                samples: std::mem::take(&mut travel).into(),
            });
        }
        channels.push(Channel {
            node: n,
            node_name: target.nodes[n as usize].name.clone(),
            path: ChannelPath::Rotation,
            step: false,
            lanes: 4,
            samples: std::mem::take(&mut rotations[k]).into(),
        });
    }
    for (tw, samples) in twists.iter().zip(twist_rotations) {
        channels.push(Channel {
            node: tw.node as u32,
            node_name: target.nodes[tw.node].name.clone(),
            path: ChannelPath::Rotation,
            step: false,
            lanes: 4,
            samples: samples.into(),
        });
    }
    channels.extend(crate::visibility::retargeted(
        source, source_map, target, target_map, clip, &binding,
    ));
    let out = ClipAsset {
        name: clip.name.clone(),
        duration: clip.duration,
        rate: clip.rate,
        source_rate: clip.source_rate,
        frames: clip.frames,
        channels,
        events: clip.events.clone(),
    };
    out.validate()?;
    Ok(Retargeted {
        clip: out,
        scale: k,
        bones: (0..HUMAN_BONE_COUNT).filter(|&b| both(b)).count() as u32,
        legs: legs.len() as u32,
        twists: twists.len() as u32,
    })
}
