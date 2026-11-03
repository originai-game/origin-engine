//! Tick-owned performance timing, with pose composition shared by generated
//! motion and og.anim's startup fallback. No generated value enters this state.

use super::*;
use og_anim::{AnimationSystem, HumanoidMap, NONE, SkeletonAsset, Trs};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Envelope {
    elapsed: f32,
    length: f32,
    blend: f32,
    release: Option<(f32, f32)>,
}

impl Envelope {
    fn new(length: f32, blend: f32) -> Self {
        Self {
            elapsed: 0.0,
            length,
            blend,
            release: None,
        }
    }
    fn weight(&self) -> f32 {
        let ramp = |t: f32| {
            let x = if self.blend == 0.0 {
                1.0
            } else {
                (t / self.blend).clamp(0.0, 1.0)
            };
            x * x * (3.0 - 2.0 * x)
        };
        if let Some((at, weight)) = self.release {
            weight * ramp(self.blend - (self.elapsed - at))
        } else {
            ramp(self.elapsed).min(ramp(self.length - self.elapsed))
        }
    }
    fn release(&mut self, blend: f32) {
        let weight = self.weight();
        self.blend = blend;
        self.release = Some((self.elapsed, weight));
    }
    fn advance(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        self.elapsed < self.release.map_or(self.length, |(at, _)| at + self.blend)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Look {
    target: [f32; 3],
    max_yaw: f32,
    max_pitch: f32,
    weight: f32,
    time: Envelope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Reaction {
    clip: u64,
    direction: [f32; 3],
    strength: f32,
    hold_start: f32,
    hold_end: f32,
    duration: f32,
    hand: String,
    hand_target: [f32; 3],
    time: Envelope,
}

impl Reaction {
    fn clip_time(&self) -> f32 {
        let t = self.time.elapsed;
        if t < self.hold_start {
            t
        } else if t < self.hold_start + self.duration {
            self.hold_start + (t - self.hold_start) % (self.hold_end - self.hold_start)
        } else {
            self.hold_end + t - self.hold_start - self.duration
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Performance {
    look: Option<Look>,
    previous_look: Option<Look>,
    reaction: Option<Reaction>,
    previous_reaction: Option<Reaction>,
}

impl Performance {
    pub fn looking(&self) -> bool {
        self.look.is_some() || self.previous_look.is_some()
    }
    pub fn reacting(&self) -> bool {
        self.reaction.is_some() || self.previous_reaction.is_some()
    }
    pub fn advance(&mut self, dt: f32) {
        for look in [&mut self.look, &mut self.previous_look] {
            if look.as_mut().is_some_and(|l| !l.time.advance(dt)) {
                *look = None;
            }
        }
        for reaction in [&mut self.reaction, &mut self.previous_reaction] {
            if reaction.as_mut().is_some_and(|r| !r.time.advance(dt)) {
                *reaction = None;
            }
        }
    }
    pub fn shift(&mut self, d: glam::DVec3) {
        for look in [&mut self.look, &mut self.previous_look]
            .into_iter()
            .flatten()
        {
            look.target = (Vec3::from_array(look.target).as_dvec3() + d)
                .as_vec3()
                .to_array();
        }
    }

    pub fn apply(
        &self,
        sys: &AnimationSystem,
        skeleton: &SkeletonAsset,
        map: &HumanoidMap,
        root: &Trs,
        nodes: &mut [Trs],
    ) {
        for reaction in [&self.previous_reaction, &self.reaction]
            .into_iter()
            .flatten()
        {
            let clip = sys
                .clip(reaction.clip)
                .expect("reaction clip registered by react");
            let mut pose = og_anim::Pose::rest(skeleton);
            og_anim::sample(
                skeleton,
                &clip,
                &og_anim::bind(skeleton, &clip),
                reaction.clip_time(),
                NONE,
                &mut pose,
            );
            let hips = map.node("hips").expect("react validated hips") as usize;
            let direction = Vec3::from_array(reaction.direction);
            if direction.length_squared() > 0.0 {
                let yaw = rotation_of(atan2(direction.x, direction.z) - heading_of(root.rotation));
                let parent = skeleton.nodes[hips].parent;
                let parent_rotation = if parent == NONE {
                    Quat::IDENTITY
                } else {
                    skeleton.model_space(&pose.nodes)[parent as usize].rotation
                };
                pose.nodes[hips].rotation =
                    parent_rotation.inverse() * yaw * parent_rotation * pose.nodes[hips].rotation;
            }
            pose.nodes[hips].translation.x = nodes[hips].translation.x;
            pose.nodes[hips].translation.z = nodes[hips].translation.z;
            let weight = reaction.strength * reaction.time.weight();
            for (node, wanted) in nodes.iter_mut().zip(&pose.nodes) {
                node.translation = node.translation.lerp(wanted.translation, weight);
                node.rotation = og_core::math::quat_slerp(node.rotation, wanted.rotation, weight);
            }
        }
        for look in [&self.previous_look, &self.look].into_iter().flatten() {
            let chain = look_chain(skeleton, map).expect("lookAt validated spine and head");
            let head = *chain.last().expect("head in look chain") as usize;
            let rest = skeleton.model_space(&skeleton.rest_pose())[head].rotation;
            let constraint = og_anim::LookAt {
                axis: rest.inverse() * Vec3::Z,
                up_axis: rest.inverse() * Vec3::Y,
                max_yaw: look.max_yaw,
                max_pitch: look.max_pitch,
                weight: look.weight * look.time.weight(),
                ..Default::default()
            };
            og_anim::look_at(
                skeleton,
                nodes,
                &chain,
                root.inverse()
                    .transform_point(Vec3::from_array(look.target)),
                Some(Vec3::Y),
                &constraint,
                &[],
            );
        }
        // Contact follows the final head pose, including the look overlay.
        for reaction in [&self.previous_reaction, &self.reaction]
            .into_iter()
            .flatten()
            .filter(|r| !r.hand.is_empty())
        {
            let chain = arm(map, &reaction.hand).expect("react validated arm");
            let head = map.node("head").expect("react validated head") as usize;
            let at = skeleton.model_space(nodes)[head]
                .transform_point(Vec3::from_array(reaction.hand_target));
            let t = reaction.clip_time();
            let blend = reaction.time.blend;
            let contact = if blend == 0.0 {
                f32::from(t >= reaction.hold_start && t <= reaction.hold_end)
            } else {
                ((t - reaction.hold_start + blend) / blend)
                    .min((reaction.hold_end + blend - t) / blend)
                    .clamp(0.0, 1.0)
            };
            og_anim::two_bone(
                skeleton,
                nodes,
                &chain,
                at,
                None,
                contact * reaction.strength * reaction.time.weight(),
                &[],
            );
        }
    }
}

fn look_chain(skeleton: &SkeletonAsset, map: &HumanoidMap) -> Result<Vec<u32>, String> {
    let (Some(head), Some(spine)) = (map.node("head"), map.node("spine")) else {
        return Err("lookAt needs mapped head and spine bones".into());
    };
    let hips = map.node("hips");
    let mut chain = vec![head];
    while *chain.last().unwrap() != spine {
        let parent = skeleton.nodes[*chain.last().unwrap() as usize].parent;
        if parent == NONE || Some(parent) == hips {
            return Err("lookAt head must descend from spine, above hips".into());
        }
        chain.push(parent);
    }
    chain.reverse();
    Ok(chain)
}

fn arm(map: &HumanoidMap, side: &str) -> Result<[u32; 3], String> {
    let bone = |part| {
        map.node(&format!("{side}{part}"))
            .ok_or_else(|| format!("reaction needs mapped {side}{part}"))
    };
    Ok([bone("UpperArm")?, bone("LowerArm")?, bone("Hand")?])
}

fn target(
    eng: &Engine,
    e: Entity,
    s: &Settings,
) -> Result<(u64, Arc<SkeletonAsset>, HumanoidMap), String> {
    let w = eng.world()?;
    let id = w.id_by_name("og.Skeleton").map_err(|e| e.to_string())?;
    let bytes = w
        .get_bytes(e, id)
        .map_err(|_| "motion performance needs og.Skeleton")?;
    let st = eng.state.borrow();
    let record = st.layouts["og.Skeleton"].decode(&bytes);
    let h = record["asset"].as_f64().unwrap_or(0.0) as u64;
    let sys = st
        .anim
        .as_ref()
        .ok_or("animation system is not installed")?;
    let (asset, map) = crate::anim::humanoid_of(eng, sys, h, s.rig, "agent")?;
    Ok((h, asset, map))
}

fn bounded(d: &Value, key: &str, default: f32, max: f32) -> Result<f32, String> {
    let v = number(d, key, default);
    if v.is_finite() && v >= 0.0 && v <= max {
        Ok(v)
    } else {
        Err(format!("desc.{key} must be finite and in 0..={max}"))
    }
}

pub fn og_motion_look_at(world: u32, entity: f64, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let s = agent_of(&eng, e)?;
        let d = parse(desc.as_deref())?;
        let blend = bounded(&d, "blend", 0.2, f32::MAX)?;
        let look = if d["target"].is_null() {
            None
        } else {
            let (_, asset, map) = target(&eng, e, &s)?;
            look_chain(&asset, &map)?;
            let duration = bounded(&d, "duration", 2.0, f32::MAX)?;
            let length = duration + 2.0 * blend;
            if !length.is_finite() {
                return Err("look duration plus blending is too long".into());
            }
            Some(Look {
                target: finite(&d["target"], "desc.target")?.to_array(),
                max_yaw: bounded(&d, "maxYaw", 1.92, std::f32::consts::PI)?,
                max_pitch: bounded(&d, "maxPitch", 0.6, std::f32::consts::FRAC_PI_2)?,
                weight: bounded(&d, "weight", 1.0, 1.0)?,
                time: Envelope::new(length, blend),
            })
        };
        let sh = shared(world);
        let mut goals = lock(&sh);
        let p = &mut goals.agents.entry(e.to_bits()).or_default().performance;
        if let Some(mut old) = p.look.take() {
            old.time.release(blend);
            p.previous_look = Some(old);
        }
        p.look = look;
        Ok(())
    })())
}

pub fn og_motion_react(world: u32, entity: f64, desc: Option<String>) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let s = agent_of(&eng, e)?;
        let d = parse(desc.as_deref())?;
        let blend = bounded(&d, "blend", 0.2, f32::MAX)?;
        let path = d["action"].as_str().unwrap_or("");
        let reaction = if path.is_empty() {
            None
        } else {
            let (target_h, target_asset, target_map) = target(&eng, e, &s)?;
            let id = og_bundle::runtime::asset_id(path);
            let action = {
                let assets = eng.assets.borrow();
                let (bytes, bundle) = assets.blob(id)?;
                bundle
                    .motion_action(id, bytes.bytes())
                    .map_err(|e| e.to_string())?
            };
            let loaded = crate::anim::load_animation(&eng, &action.source)?;
            let source_h = loaded["skeleton"].as_f64().unwrap() as u64;
            let clip = loaded["clips"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == action.clip)
                .ok_or_else(|| format!("{} has no clip '{}'", action.source, action.clip))?;
            let length = clip["duration"].as_f64().unwrap() as f32;
            if action.hold_end >= length {
                return Err("motion-action holdEnd must precede the clip's end".into());
            }
            let source_rig = if action.rig.is_empty() {
                0
            } else {
                crate::anim::load_rig(&eng, &action.rig)?.as_f64().unwrap() as u64
            };
            let retargeted = crate::anim::retarget(
                &eng,
                &json!({"clip": clip["clip"], "source": source_h as f64, "target": target_h as f64, "sourceRig": source_rig as f64, "targetRig": s.rig as f64}),
            )?;
            let mut hand_target = Vec3::from_array(action.hand_target);
            if !action.hand.is_empty() {
                arm(&target_map, &action.hand)?;
                let sys = eng.state.borrow().anim.clone().unwrap();
                let (source, source_map) =
                    crate::anim::humanoid_of(&eng, &sys, source_h, source_rig, "source")?;
                let head =
                    |map: &HumanoidMap| map.node("head").ok_or("hand contact needs a mapped head");
                let src = source.model_space(&source.rest_pose());
                let dst = target_asset.model_space(&target_asset.rest_pose());
                let sh = head(&source_map)? as usize;
                let th = head(&target_map)? as usize;
                // Preserve the head-local contact across different bone frames
                // and proportions, using the arm's length as its scale.
                let length = |pose: &[Trs], chain: [u32; 3]| {
                    (pose[chain[1] as usize].translation - pose[chain[0] as usize].translation)
                        .length()
                        + (pose[chain[2] as usize].translation
                            - pose[chain[1] as usize].translation)
                            .length()
                };
                let ratio = length(&dst, arm(&target_map, &action.hand)?)
                    / length(&src, arm(&source_map, &action.hand)?);
                hand_target = (dst[th].rotation.inverse() * src[sh].rotation * hand_target * ratio)
                    / dst[th].scale;
            }
            let duration = bounded(&d, "duration", 1.0, f32::MAX)?;
            let total = action.hold_start + duration + length - action.hold_end;
            if !total.is_finite() {
                return Err("reaction duration is too long".into());
            }
            Some(Reaction {
                clip: retargeted["clip"].as_f64().unwrap() as u64,
                direction: finite(&d["direction"], "desc.direction")?.to_array(),
                strength: bounded(&d, "strength", 1.0, 1.0)?,
                hold_start: action.hold_start,
                hold_end: action.hold_end,
                duration,
                hand: action.hand,
                hand_target: hand_target.to_array(),
                time: Envelope::new(total, blend),
            })
        };
        let sh = shared(world);
        let mut goals = lock(&sh);
        let p = &mut goals.agents.entry(e.to_bits()).or_default().performance;
        if let Some(mut old) = p.reaction.take() {
            old.time.release(blend);
            p.previous_reaction = Some(old);
        }
        p.reaction = reaction;
        Ok(())
    })())
}
