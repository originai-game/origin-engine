//! Graph and player clip playback.

use super::*;

/// A clip leaf's playback over this tick, from `phase0` to `phase1`.
#[allow(clippy::too_many_arguments)]
fn phase_leaf<'a>(
    handle: u64,
    clip: &'a ClipAsset,
    binding: &'a [u32],
    looping: bool,
    before: Phase,
    after: Phase,
    wraps: i32,
    weight: f32,
) -> Leaf<'a> {
    let len = clip.duration;
    let at = |p: Phase| {
        if looping {
            p.frac * len
        } else if p.cycles >= 1 {
            len
        } else {
            p.frac * len
        }
    };
    Leaf {
        handle,
        clip,
        binding,
        t0: at(before),
        t1: at(after),
        wraps: if looping { wraps } else { 0 },
        weight,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn eval_graph(
    sh: &Shared,
    e: Entity,
    skeleton_handle: u64,
    skel: &SkeletonAsset,
    graph: &GraphAsset,
    comp: &mut AnimationGraph,
    gs: &mut GraphState,
    root: Option<&RootMotion>,
    mesh: u32,
    dt: f32,
    events: &mut Vec<Fired>,
    stance: &mut Stance,
) -> Result<(Pose, Trs)> {
    if gs.asset != comp.asset || gs.layers.len() != graph.layers.len() {
        gs.asset = comp.asset;
        gs.layers = graph.layers.iter().map(LayerState::entry).collect();
    }
    let params = comp.params();
    let speed = comp.speed.max(0.0);
    let graph_weight = comp.weight.clamp(0.0, 1.0);
    let mut pose = Pose::rest(skel);
    let mut delta = Trs::IDENTITY;
    // Each sync group's phase this tick (before, after, wraps), set by the
    // first layer whose current state is in it.
    let mut leaders: Vec<Option<(Phase, Phase, i32)>> = vec![None; graph.sync_groups.len()];
    for (li, layer) in graph.layers.iter().enumerate() {
        let ls = &mut gs.layers[li];
        ls.transition(layer, &params);
        let layer_weight = layer.weight(&params);
        // The cross-fade's progress after this tick.
        let alpha = ls.fade_alpha(dt);
        let base = li == 0;
        let leaf_root = if base { root } else { None };
        let mut layer_blend = Blend::new();
        let mut reference = Blend::new();
        let mut run_state = |state: usize,
                             phase: &mut Phase,
                             w: f32,
                             follow: Option<(Phase, Phase, i32)>|
         -> Result<(Phase, i32)> {
            let st = &layer.states[state];
            let mut leaves = Vec::new();
            graph.leaves(st.motion, &params, 1.0, &mut leaves);
            let mut duration = 0.0f32;
            for &(n, lw) in &leaves {
                if let Motion::Clip {
                    clip, speed: cs, ..
                } = graph.nodes[n].motion
                {
                    let c = sh
                        .clips
                        .get(&clip)
                        .ok_or_else(|| missing(e, "clip", clip))?;
                    duration += lw * c.duration / cs;
                }
            }
            let (before, wraps) = match follow {
                // A follower plays at its group leader's phase.
                Some((b, a, k)) => {
                    *phase = a;
                    (b, k)
                }
                None => {
                    let before = *phase;
                    let wraps = if duration > 0.0 {
                        phase.advance(dt * speed * st.speed / duration)
                    } else {
                        0
                    };
                    (before, wraps)
                }
            };
            for &(n, lw) in &leaves {
                let Motion::Clip {
                    clip,
                    looping,
                    mirror,
                    ..
                } = graph.nodes[n].motion
                else {
                    continue;
                };
                let mirrored = if mirror {
                    Some(
                        sh.mirrored(skeleton_handle, clip)?
                            .ok_or_else(|| missing(e, "clip", clip))?,
                    )
                } else {
                    None
                };
                let c = sh
                    .clips
                    .get(&clip)
                    .ok_or_else(|| missing(e, "clip", clip))?;
                let b = sh
                    .binding(skeleton_handle, clip)
                    .ok_or_else(|| missing(e, "clip", clip))?;
                let (c, b): (&ClipAsset, &[u32]) = match &mirrored {
                    Some(m) => (&m.0, &m.1),
                    None => (c, &b),
                };
                let leaf = phase_leaf(
                    clip,
                    c,
                    b,
                    looping,
                    before,
                    *phase,
                    wraps,
                    lw * w * layer_weight * graph_weight,
                );
                leaf.events(events);
                let d = if base {
                    stance.push(((clip, mirror), leaf.t1, lw * w * layer_weight));
                    leaf.delta(skel, leaf_root)
                } else {
                    Trs::IDENTITY
                };
                layer_blend.push(leaf.pose(skel, leaf_root, mesh), d, lw * w);
                if layer.mode == LayerMode::Additive {
                    reference.push(leaf.reference(skel, mesh), Trs::IDENTITY, lw * w);
                }
            }
            Ok((before, wraps))
        };
        let (cur, mut cur_phase) = (ls.state as usize, ls.phase);
        if ls.prev != NONE && alpha < 1.0 {
            let (prev, mut prev_phase) = (ls.prev as usize, ls.prev_phase);
            run_state(prev, &mut prev_phase, 1.0 - alpha, None)?;
            ls.prev_phase = prev_phase;
        }
        let group = layer.states[cur].sync;
        let follow = group.and_then(|g| leaders[g]);
        let (before, wraps) = run_state(cur, &mut cur_phase, alpha, follow)?;
        if let Some(g) = group
            && leaders[g].is_none()
        {
            leaders[g] = Some((before, cur_phase, wraps));
        }
        ls.phase = cur_phase;
        ls.time += dt * speed;
        ls.settle(alpha);
        let layer_pose = layer_blend.pose.unwrap_or_else(|| Pose::rest(skel));
        let mask = layer.node_weights(skel);
        match (layer.mode, &mask) {
            (LayerMode::Override, None) => pose.lerp_to(&layer_pose, layer_weight),
            (LayerMode::Override, Some(m)) => pose.lerp_to_masked(&layer_pose, layer_weight, m),
            (LayerMode::Additive, _) => {
                let r = reference.pose.unwrap_or_else(|| Pose::rest(skel));
                match &mask {
                    None => pose.add(&layer_pose, &r, layer_weight),
                    Some(m) => pose.add_masked(&layer_pose, &r, layer_weight, m),
                }
            }
        }
        if base {
            delta = Trs::IDENTITY.lerp(&layer_blend.delta, layer_weight);
        }
    }
    if let Some(l) = gs.layers.first() {
        comp.state = l.state;
        comp.state_time = l.time;
    }
    Ok((pose, delta))
}

/// Advances clip time `t` by `d` over a clip of `len`: the new time and the wraps.
fn advance_time(t: f32, d: f32, len: f32, looping: bool) -> (f32, i32) {
    let raw = t + d;
    if looping && len > 0.0 {
        let k = (raw / len).floor();
        let mut t1 = raw - k * len;
        if t1 >= len {
            t1 = 0.0;
        }
        if t1 < 0.0 {
            t1 = 0.0;
        }
        (t1, k as i32)
    } else {
        (raw.clamp(0.0, len.max(0.0)), 0)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn eval_player(
    sh: &Shared,
    e: Entity,
    skeleton_handle: u64,
    skel: &SkeletonAsset,
    comp: &mut AnimationPlayer,
    ps: &mut PlayerState,
    root: Option<&RootMotion>,
    mesh: u32,
    dt: f32,
    step_default: f32,
    events: &mut Vec<Fired>,
    stance: &mut Stance,
) -> Result<Option<(Pose, Trs)>> {
    let seek = comp.time != ps.written;
    // The player's own step, else the art contract's (never written back:
    // the component keeps 0, so a look without one plays smooth again).
    let step_fps = if comp.step_fps > 0.0 {
        comp.step_fps
    } else {
        step_default
    };
    if comp.clip != ps.clip {
        if ps.clip != 0 && comp.fade > 0.0 && sh.clips.contains_key(&ps.clip) {
            ps.prev_clip = ps.clip;
            ps.prev_time = ps.time;
            ps.prev_speed = ps.speed;
            ps.prev_looping = ps.looping;
            ps.fade = 0.0;
            ps.fade_duration = comp.fade;
        } else {
            ps.prev_clip = 0;
        }
        ps.clip = comp.clip;
        ps.time = if seek { comp.time } else { 0.0 };
    } else if seek {
        ps.time = comp.time;
    }
    ps.speed = comp.speed;
    ps.looping = comp.looping;
    if ps.clip == 0 {
        ps.prev_clip = 0;
        ps.written = comp.time;
        return Ok(None);
    }
    let clip = sh
        .clips
        .get(&ps.clip)
        .ok_or_else(|| missing(e, "og.AnimationPlayer.clip", ps.clip))?;
    let alpha = if ps.prev_clip != 0 {
        ps.fade += dt;
        if ps.fade_duration > 0.0 && ps.fade < ps.fade_duration {
            ps.fade / ps.fade_duration
        } else {
            1.0
        }
    } else {
        1.0
    };
    // The player decides `weight` of the pose over what the graph left.
    let wt = comp.weight.clamp(0.0, 1.0);
    for s in stance.iter_mut() {
        s.2 *= 1.0 - wt;
    }
    let mut blend = Blend::new();
    if ps.prev_clip != 0 && alpha < 1.0 {
        let pc = sh
            .clips
            .get(&ps.prev_clip)
            .ok_or_else(|| missing(e, "clip", ps.prev_clip))?;
        let b = sh
            .binding(skeleton_handle, ps.prev_clip)
            .ok_or_else(|| missing(e, "clip", ps.prev_clip))?;
        let (t1, wraps) = advance_time(
            ps.prev_time,
            dt * ps.prev_speed,
            pc.duration,
            ps.prev_looping,
        );
        let leaf = Leaf {
            handle: ps.prev_clip,
            clip: pc,
            binding: &b,
            t0: ps.prev_time,
            t1,
            wraps,
            weight: (1.0 - alpha) * comp.weight,
        };
        leaf.events(events);
        let held = stepped(t1, step_fps, pc.duration, ps.prev_looping);
        stance.push(((ps.prev_clip, false), held, (1.0 - alpha) * wt));
        blend.push(
            leaf.pose_at(skel, root, mesh, held),
            leaf.delta(skel, root),
            1.0 - alpha,
        );
        ps.prev_time = t1;
    }
    if alpha >= 1.0 {
        ps.prev_clip = 0;
    }
    let b = sh
        .binding(skeleton_handle, ps.clip)
        .ok_or_else(|| missing(e, "clip", ps.clip))?;
    let (t1, wraps) = advance_time(ps.time, dt * comp.speed, clip.duration, comp.looping);
    let leaf = Leaf {
        handle: ps.clip,
        clip,
        binding: &b,
        t0: ps.time,
        t1,
        wraps,
        weight: alpha * comp.weight,
    };
    leaf.events(events);
    let held = stepped(t1, step_fps, clip.duration, comp.looping);
    stance.push(((ps.clip, false), held, alpha * wt));
    blend.push(
        leaf.pose_at(skel, root, mesh, held),
        leaf.delta(skel, root),
        alpha,
    );
    ps.time = t1;
    ps.written = t1;
    comp.time = t1;
    comp.finished = !comp.looping
        && ((comp.speed >= 0.0 && t1 >= clip.duration) || (comp.speed < 0.0 && t1 <= 0.0));
    Ok(blend.pose.map(|p| (p, blend.delta)))
}
