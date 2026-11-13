//! A generated pose drawn: re-rooted onto the entity, retargeted to its
//! rig, its feet planted, and written as the skin's joint palette.

use super::*;

/// SOMA's head joint.
pub(crate) const HEAD: usize = 6;
/// The drawn hips follow the generated hips' own travel and are drawn
/// back to the Transform with this half-life, s, never farther from it
/// than [`MAX_OFFSET`] m (times the character's scale): within a stride
/// the hips speed up and slow down, as a walker's do, over feet that stay
/// put, while the Transform moves at the simulation's even speed.
pub(crate) const OFFSET_HALF_LIFE: f32 = 0.35;
pub(crate) const MAX_OFFSET: f32 = 0.15;
/// How fast the drawn hips may move to keep the planted feet still, m/s.
pub(crate) const FOLLOW_SPEED: f32 = 1.5;

/// Moves the drawn root's offset on by `dt` s: the generated hips' travel
/// since the last frame (turned from the model's heading to the
/// Transform's) less the Transform's, eased back to none and bounded. A
/// jump clip (the Transform carries it), a cut in the generated motion or
/// the first frame draws the hips on the Transform.
pub(crate) fn follow_hips(
    (offset, followed): (&mut Vec3, &mut Option<(Vec3, f32)>),
    a: &Seen,
    (prev, next, t): (&Frame, &Frame, f32),
    velocity: Vec3,
    cut: bool,
    dt: f32,
) {
    let at = |j: usize| {
        let (u, v) = (prev.pose.positions[j], next.pose.positions[j]);
        Vec3::new(u[0] + (v[0] - u[0]) * t, 0.0, u[2] + (v[2] - u[2]) * t)
    };
    let hips = at(0);
    let h = heading(at(RIGHT_LEG).to_array(), at(LEFT_LEG).to_array());
    match *followed {
        Some((last, last_h)) if !cut && hips.distance(last) < 0.3 => {
            let moved = rotation_of(heading_of(a.rotation) - last_h) * (hips - last);
            let sim = Vec3::new(velocity.x, 0.0, velocity.z) * dt;
            *offset += moved * a.scale - sim;
            let k = og_core::math::exp(-dt * std::f32::consts::LN_2 / OFFSET_HALF_LIFE);
            *offset = (*offset * k).clamp_length_max(MAX_OFFSET * a.scale);
        }
        _ => *offset = Vec3::ZERO,
    }
    *followed = Some((hips, h));
}

/// `pose`'s local transforms re-rooted: the hips over the origin at their
/// height, facing +Z, and the lean (hips to head) they have.
pub(crate) fn rerooted(skeleton: &Skeleton, asset: &SkeletonAsset, pose: &Pose) -> (Vec<Trs>, f32) {
    let h = heading(pose.positions[RIGHT_LEG], pose.positions[LEFT_LEG]);
    let mut local = skeleton.local(pose);
    let undo = rotation_of(-h);
    local[0].translation = Vec3::new(0.0, local[0].translation.y, 0.0);
    local[0].rotation = (undo * local[0].rotation).normalize();
    // Measured on the transforms as drawn (forward kinematics), not on the
    // model's positions.
    let m = asset.model_space(&local);
    let lean = lean_of(m[HEAD].translation - m[0].translation);
    (local, lean)
}

pub(crate) fn palette(
    r: &Ready,
    a: &Seen,
    s: &mut Shown,
    goals: &Goals,
    sys: &og_anim::AnimationSystem,
    dt: f32,
) -> Option<Vec<Mat4>> {
    let (target, map) = s.target.as_ref()?;
    let prev = s.prev?;
    let next = s.next.unwrap_or(prev);
    let (a0, l0) = rerooted(&r.soma_skeleton, &r.soma, &prev.pose);
    let (a1, l1) = rerooted(&r.soma_skeleton, &r.soma, &next.pose);
    s.lean = Some(l0 + (l1 - l0) * s.phase.clamp(0.0, 1.0));
    let mut frames = [a0, a1];
    // In a jump the Transform carries the simulation's elevation: the hips
    // keep only their height over the lowest foot's clearance.
    if s.clip.is_some() {
        for (local, f) in frames.iter_mut().zip([&prev, &next]) {
            let foot = f.pose.positions[LEFT_ANKLE][1].min(f.pose.positions[RIGHT_ANKLE][1]);
            local[0].translation.y =
                flight::local_height(local[0].translation.y, foot, r.clearance);
        }
    }
    let clip = og_anim::motion::soma::clip("motion", &r.soma, &frames);
    let out = og_anim::retarget(
        &r.soma,
        &r.soma_map,
        target,
        map,
        &clip,
        &RetargetOptions::default(),
    )
    .ok()?;
    let mut pose = AnimPose::rest(target);
    let binding = og_anim::bind(target, &out.clip);
    og_anim::sample(target, &out.clip, &binding, s.phase / RATE, NONE, &mut pose);
    // The feet's contacts from the generated pose itself: each ankle's
    // height over its rest height in the model's world and its speed
    // there (the model's own time), with hysteresis. Never in the air.
    let jump = goals
        .agents
        .get(&a.entity)
        .and_then(|g| g.current)
        .and_then(|c| c.jump)
        .map(|j| j.stage);
    let airborne = jump == Some(JumpStage::Air);
    // Holding a pose in place: no goal, or a pose goal, with the Transform
    // standing still.
    let holding = jump.is_none()
        && Vec3::new(s.velocity.x, 0.0, s.velocity.z).length() < 0.05
        && goals
            .agents
            .get(&a.entity)
            .is_none_or(|g| g.current.is_none_or(|c| c.goal.kind == Kind::Pose));
    let rest = r.soma.model_space(&r.soma.rest_pose())[LEFT_ANKLE]
        .translation
        .y;
    let t = s.phase.clamp(0.0, 1.0);
    let mut contacts = [0.0; 4];
    for (k, ankle) in [LEFT_ANKLE, RIGHT_ANKLE].into_iter().enumerate() {
        let (u, v) = (prev.pose.positions[ankle], next.pose.positions[ankle]);
        let height = u[1] + (v[1] - u[1]) * t - rest;
        let speed = Vec3::from_array(v).distance(Vec3::from_array(u)) * RATE;
        let down = s.feet[k].step(height, speed, holding) && !airborne;
        if airborne {
            s.feet[k] = Foot::default();
        }
        contacts[2 * k] = if down { 1.0 } else { 0.0 };
    }
    let root = Trs::new(a.position, a.rotation, Vec3::splat(a.scale));
    let cut = s.clip.is_some();
    let (t, velocity) = (s.phase.clamp(0.0, 1.0), s.velocity);
    follow_hips(
        (&mut s.offset, &mut s.followed),
        a,
        (&prev, &next, t),
        velocity,
        cut,
        dt,
    );
    // The body is carried over the feet it stands on: the hips move by
    // how far the generated ankles of the planted feet have drifted from
    // where those feet stand (at most FOLLOW_SPEED), so the drawn stride
    // keeps its feet still and the offset, bounded and eased back, takes
    // up the difference with the Transform.
    if !airborne {
        let model = target.model_space(&pose.nodes);
        let drift: Vec<Vec3> = s
            .planter
            .planted()
            .map(|(leg, at)| {
                let ankle = model[leg.chain[2] as usize].translation;
                let g = root.transform_point(ankle + a.rotation.inverse() * s.offset / a.scale);
                Vec3::new(at.x - g.x, 0.0, at.z - g.z)
            })
            .collect();
        if !drift.is_empty() {
            let mean = drift.iter().copied().sum::<Vec3>() / drift.len() as f32;
            s.offset += mean.clamp_length_max(FOLLOW_SPEED * a.scale * dt);
            s.offset = s.offset.clamp_length_max(MAX_OFFSET * a.scale);
        }
    }
    if let Some(hips) = map.node("hips")
        && s.offset != Vec3::ZERO
    {
        let local = a.rotation.inverse() * s.offset / a.scale;
        let parent = target.nodes[hips as usize].parent;
        let local = if parent == NONE {
            local
        } else {
            let p = target.model_space(&pose.nodes)[parent as usize];
            p.rotation.inverse() * local / p.scale
        };
        pose.nodes[hips as usize].translation += local;
    }
    perform_and_plant(
        &mut s.planter,
        goals.agents.get(&a.entity).map(|g| &g.performance),
        sys,
        (target, map),
        &root,
        &mut pose.nodes,
        (contacts, (!airborne).then_some(a.position.y), dt),
    );
    let model = target.model_space(&pose.nodes);
    s.drawn_lean = map.node("hips").zip(map.node("head")).map(|(hips, head)| {
        lean_of(model[head as usize].translation - model[hips as usize].translation)
    });
    for (k, (ankle, contact)) in [(map.nodes[16], 0), (map.nodes[20], 2)]
        .into_iter()
        .enumerate()
    {
        let down = contacts[contact] > og_anim::motion::soma::CONTACT;
        if ankle == NONE || !down {
            s.planted[k] = None;
            continue;
        }
        let at = root.transform_point(model[ankle as usize].translation);
        if let Some(was) = s.planted[k] {
            let d = (at - was).length();
            s.slide = (s.slide.0 + d, s.slide.1 + 1, s.slide.2.max(d));
        }
        s.planted[k] = Some(at);
    }
    let rest = s.planter.rest();
    let segment = segment_of(goals.agents.get(&a.entity), a.settings.drive);
    for (k, ankle) in [map.nodes[16], map.nodes[20]].into_iter().enumerate() {
        if ankle == NONE {
            continue;
        }
        let at = root.transform_point(model[ankle as usize].translation);
        let clip = s.clip.is_some();
        s.footing.record(
            k,
            at,
            a.position.y + rest[k] * a.scale,
            (segment, airborne, clip),
            dt,
        );
    }
    let (l, r) = (map.nodes[14], map.nodes[18]);
    if l != NONE && r != NONE {
        let at = |n: u32| model[n as usize].translation.to_array();
        s.facing = heading(at(r), at(l));
        let forward = Vec3::new(
            og_core::math::sin(s.facing),
            0.0,
            og_core::math::cos(s.facing),
        );
        let (la, ra) = (map.nodes[16], map.nodes[20]);
        s.feet_ahead = map
            .node("hips")
            .filter(|_| la != NONE && ra != NONE)
            .map(|hips| {
                let mid = (model[la as usize].translation + model[ra as usize].translation) * 0.5;
                (mid - model[hips as usize].translation).dot(forward)
            });
    }
    let mut m = Vec::new();
    target.palette(a.skin, &pose.nodes, &mut m);
    (!m.is_empty()).then_some(m)
}

/// The generated pose's layers, in priority order (`docs/design.md` §2.7,
/// "Priorities"): the authored performance over the generated pose (the
/// reaction, the look, the hand contact on the head), then the feet planted
/// on the generated contacts, which outrank both. A foot the reaction's own
/// clip steps ([`Performance::stepping`]) is the clip's to move and is not
/// planted.
pub(crate) fn perform_and_plant(
    planter: &mut Planter,
    performance: Option<&super::performance::Performance>,
    sys: &og_anim::AnimationSystem,
    (target, map): (&SkeletonAsset, &HumanoidMap),
    root: &Trs,
    nodes: &mut [Trs],
    (mut contacts, ground, dt): ([f32; 4], Option<f32>, f32),
) {
    if let Some(performance) = performance {
        performance.apply(sys, target, map, root, nodes);
        let stepping = performance.stepping(sys, target, map, root, [map.nodes[16], map.nodes[20]]);
        for (k, step) in stepping.into_iter().enumerate() {
            if step {
                contacts[2 * k] = 0.0;
                contacts[2 * k + 1] = 0.0;
            }
        }
    }
    planter.step(target, nodes, root, &contacts, ground, dt);
}

/// Compose authored performance on the animation pose during model startup.
pub(crate) fn fallback(
    eng: &Engine,
    p: &mut Presenter,
    agents: &[Seen],
    goals: &Goals,
    sys: Option<&og_anim::AnimationSystem>,
    poses: &BTreeMap<u64, Vec<Trs>>,
) {
    let Some(sys) = sys else {
        return;
    };
    for a in agents {
        if p.palettes.contains_key(&a.entity) {
            continue;
        }
        let Some(agent) = goals.agents.get(&a.entity) else {
            continue;
        };
        if !agent.performance.looking() && !agent.performance.reacting() {
            continue;
        }
        let Ok((asset, map)) =
            crate::anim::humanoid_of(eng, sys, a.skeleton, a.settings.rig, "agent")
        else {
            continue;
        };
        let Some(mut nodes) = poses.get(&a.entity).cloned() else {
            continue;
        };
        let root = Trs::new(a.position, a.rotation, Vec3::splat(a.scale));
        agent
            .performance
            .apply(sys, &asset, &map, &root, &mut nodes);
        let mut palette = Vec::new();
        asset.palette(a.skin, &nodes, &mut palette);
        p.palettes.insert(a.entity, palette);
    }
}
