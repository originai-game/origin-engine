//! Joints between parts, their pose, clips over time and how parts seat at rest.

use super::*;

/// How a child moves on its parent: a zero-value desc.
///
/// The zero value is a revolute joint about the world origin's +Z with no
/// name and no range, which [`Model::joint`] refuses until a name and a
/// range are given.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JointDesc {
    /// Unique among the model's joints.
    pub name: String,
    /// Revolute, continuous, prismatic, fixed or floating.
    pub kind: JointKind,
    /// The pivot, in the world at rest: revolute and continuous joints turn
    /// about `axis` through its origin; the joint's node sits here.
    pub on: Frame,
    /// The axis, a world direction; zero is `on`'s +Z (its normal).
    /// Positive values turn by the right hand about it, or slide along it.
    /// Fixed and floating joints take none.
    pub axis: DVec3,
    /// `(lower, upper)` containing 0: radians for turns, the model's unit
    /// for slides. Revolute and prismatic joints need one; the others,
    /// and followers, take none.
    pub range: Option<(f64, f64)>,
    /// This joint's value is `multiplier` × another's (radians or units
    /// per radian or unit, across kinds).
    pub mimic: Option<Mimic>,
}

#[derive(Clone, Debug)]
pub(crate) struct JointRec {
    pub(crate) child: Id,
    pub(crate) parent: Id,
    pub(crate) desc: JointDesc,
    /// Unit world axis (none for fixed and floating).
    pub(crate) axis: Option<DVec3>,
    pub(crate) value: f64,
    /// The parent's own space seen from the pivot when the joint was
    /// declared (`on⁻¹ ∘ parent placement`): [`Model::seating`] measures
    /// how far the parent has moved off it since.
    pub(crate) seat: Frame,
}

/// How a joint holds, measured at rest in the model's unit
/// ([`Model::seating`]). Measurements only: no threshold.
#[derive(Clone, Debug, PartialEq)]
pub struct Seating {
    /// The joint.
    pub joint: String,
    /// The node it moves.
    pub child: String,
    /// The node it hangs on.
    pub parent: String,
    /// Its value now (a follower's from its leader).
    pub value: f64,
    /// From the pivot's origin to the nearest point of the parent's
    /// parts (those under it but not under the child); `None` when the
    /// parent has no triangles.
    pub pivot_to_parent: Option<f64>,
    /// From the pivot's origin to the nearest point of the child's parts.
    pub pivot_to_child: Option<f64>,
    /// How far the parent's own space has moved from the pivot since the
    /// joint was declared: a parent moved (`at`, `transform`) without its
    /// child leaves the pivot, and the child, where they were.
    pub parent_moved: f64,
    /// How far it has turned since, in radians.
    pub parent_turned: f64,
}

/// One keyframe of a clip ([`Model::clip`]): a time in seconds and the
/// joints' values then, as [`Model::pose`] takes them (radians, or the
/// model's unit for a slide).
pub type Key = (f64, Vec<(String, f64)>);

/// A named motion of the model's joints over time.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Clip {
    pub(crate) name: String,
    pub(crate) keys: Vec<Key>,
}

impl Model {
    /// The jointed node `id` moves with: its nearest jointed
    /// ancestor-or-self, or `None` for the root body.
    pub(crate) fn body(&self, id: Id) -> Option<Id> {
        let mut at = Some(id);
        while let Some(i) = at {
            if self.joints.iter().any(|j| j.child == i) {
                return Some(i);
            }
            at = self.nodes[i.0 as usize].parent;
        }
        None
    }

    pub(crate) fn joint_index(&self, name: &str) -> Result<usize, ModelError> {
        self.joints
            .iter()
            .position(|j| j.desc.name == name)
            .ok_or_else(|| {
                ModelError::NoSuchJoint(
                    name.into(),
                    self.joints.iter().map(|j| j.desc.name.clone()).collect(),
                )
            })
    }

    /// Whether the joint at `k` turns without end (continuous, or a turn
    /// following one).
    pub(crate) fn endless(&self, k: usize) -> bool {
        let j = &self.joints[k];
        match &j.desc.mimic {
            None => j.desc.kind == JointKind::Continuous,
            Some(m) => {
                j.desc.kind != JointKind::Prismatic
                    && self.joint_index(&m.joint).is_ok_and(|l| self.endless(l))
            }
        }
    }

    /// Declares how `child` (a part or a group) moves on `parent` (a part,
    /// or a group). It does not move the child: place it first, at rest.
    /// A second joint on one child must hang on the same parent; it rides
    /// on the first one's motion.
    pub fn joint(&mut self, child: Id, parent: Id, d: &JointDesc) -> Result<(), ModelError> {
        let name = &d.name;
        let err = |why: &str| Err(ModelError::Joint(name.clone(), why.into()));
        if name.is_empty() {
            return Err(ModelError::EmptyName);
        }
        let cn = self.node(child)?.name.clone();
        let pn = self.node(parent)?.name.clone();
        if self.joints.iter().any(|j| &j.desc.name == name) {
            return err("a joint already has this name");
        }
        finite_frame(&d.on, "the joint's frame `on`")?;
        if !d.axis.is_finite() {
            return Err(ModelError::BadValue(format!("joint {name:?}'s axis")));
        }
        if self.under(parent, child) {
            return err(&format!(
                "its parent {pn:?} is {cn:?} or inside it; hang it on the part it moves on"
            ));
        }
        if let Some(j) = self.joints.iter().find(|j| j.child == child)
            && j.parent != parent
        {
            return err(&format!(
                "{cn:?} already hangs on {:?}; every joint of one child hangs on the same parent",
                self.nodes[j.parent.0 as usize].name
            ));
        }
        // No cycle: walk up from the parent's body through the joints.
        let mut b = self.body(parent);
        let mut seen = BTreeSet::new();
        while let Some(x) = b {
            let above = self.joints.iter().find(|j| j.child == x).map(|j| j.parent);
            if self.under(x, child) || above.is_some_and(|p| self.under(p, child)) {
                return err(&format!("{pn:?} already hangs on {cn:?}: a cycle"));
            }
            if !seen.insert(x) {
                break;
            }
            b = above.and_then(|p| self.body(p));
        }
        let kind = d.kind;
        let free = matches!(kind, JointKind::Fixed | JointKind::Floating);
        if free && (d.axis != DVec3::ZERO || d.range.is_some() || d.mimic.is_some()) {
            return err("fixed and floating joints take no axis, range or mimic");
        }
        let axis = if free {
            None
        } else if d.axis == DVec3::ZERO {
            Some(d.on.z)
        } else {
            Some(
                d.axis
                    .try_normalize()
                    .ok_or_else(|| ModelError::Joint(name.clone(), "its axis is zero".into()))?,
            )
        };
        if let Some(m) = &d.mimic {
            if d.range.is_some() {
                return err("a follower takes no range: its range is its leader's × multiplier");
            }
            if !m.multiplier.is_finite() || m.multiplier == 0.0 {
                return err("the mimic's multiplier must be finite and not 0");
            }
            let l = self.joint_index(&m.joint)?;
            if matches!(
                self.joints[l].desc.kind,
                JointKind::Fixed | JointKind::Floating
            ) {
                return err("it follows a fixed or floating joint, which has no value");
            }
            if kind == JointKind::Prismatic && self.endless(l) {
                return err("a slide that follows an endless turn would slide without end");
            }
        } else {
            match (kind, d.range) {
                (JointKind::Revolute | JointKind::Prismatic, None) => {
                    return err(
                        "a revolute or prismatic joint needs a range (continuous turns without end)",
                    );
                }
                (JointKind::Continuous, Some(_)) => {
                    return err("a continuous joint turns without end: it takes no range");
                }
                (_, Some((lo, hi))) => {
                    if !(lo.is_finite() && hi.is_finite()) {
                        return Err(ModelError::BadValue(format!("joint {name:?}'s range")));
                    }
                    if !(lo < hi && lo <= 0.0 && 0.0 <= hi) {
                        return err("its range must run from low to high and contain 0, the rest");
                    }
                }
                _ => {}
            }
        }
        let seat =
            d.on.inverse()
                .compose(&self.nodes[parent.0 as usize].placement);
        self.joints.push(JointRec {
            child,
            parent,
            desc: d.clone(),
            axis,
            value: 0.0,
            seat,
        });
        Ok(())
    }

    /// Poses joint `name` at `value` (radians for turns, the unit for
    /// slides), within its range. Followers move with it; 0 is the rest.
    /// Export, preview and [`Model::measure_posed`] show the pose; frames
    /// are not read off it ([`ModelError::Posed`]).
    pub fn pose(&mut self, name: &str, value: f64) -> Result<(), ModelError> {
        let k = self.joint_index(name)?;
        if !value.is_finite() {
            return Err(ModelError::BadValue(format!("pose {name:?}")));
        }
        let j = &self.joints[k];
        if let Some(m) = &j.desc.mimic {
            return Err(ModelError::Joint(
                name.into(),
                format!("it follows {:?}: pose that one", m.joint),
            ));
        }
        if matches!(j.desc.kind, JointKind::Fixed | JointKind::Floating) {
            return Err(ModelError::Joint(
                name.into(),
                "a fixed or floating joint has no value to pose".into(),
            ));
        }
        if let Some((lo, hi)) = j.desc.range {
            let slack = 1e-9 * lo.abs().max(hi.abs()).max(1.0);
            if !(lo - slack <= value && value <= hi + slack) {
                return Err(ModelError::OutOfRange {
                    joint: name.into(),
                    value,
                    range: (lo, hi),
                });
            }
        }
        self.joints[k].value = value;
        Ok(())
    }

    /// The joints' names, in declaration order.
    pub fn joint_names(&self) -> Vec<String> {
        self.joints.iter().map(|j| j.desc.name.clone()).collect()
    }

    /// The value of the joint at `k`: its own, or a follower's leader's ×
    /// its multiplier.
    pub(crate) fn value_of(&self, k: usize) -> f64 {
        let j = &self.joints[k];
        match &j.desc.mimic {
            None => j.value,
            Some(m) => self
                .joint_index(&m.joint)
                .map_or(0.0, |l| self.value_of(l) * m.multiplier),
        }
    }

    /// The pose: every joint that can be posed (not a follower, fixed or
    /// floating) with its value, in declaration order.
    pub fn pose_values(&self) -> Vec<(String, f64)> {
        self.joints
            .iter()
            .filter(|j| {
                j.desc.mimic.is_none()
                    && !matches!(j.desc.kind, JointKind::Fixed | JointKind::Floating)
            })
            .map(|j| (j.desc.name.clone(), j.value))
            .collect()
    }

    /// Whether any joint has a value other than 0: export writes OBJ and
    /// STL posed then ([`Model::obj`]).
    pub fn posed(&self) -> bool {
        self.joints.iter().any(|j| j.value != 0.0)
    }

    /// Declares a clip: a named motion of joints over time, written to
    /// the GLB as an animation (`crate::tools::glb_clips`). Each key is a
    /// time in seconds and the values of the joints it moves; times rise
    /// from 0 or more. A joint a key leaves out keeps its own track: only
    /// the keys that name it move it, and it holds its first and last
    /// value outside them. Every value is checked as [`Model::pose`]
    /// checks it (the joint exists, is not a follower, fixed or floating,
    /// and the value is in its range); the model's pose is not changed.
    pub fn clip(&mut self, name: &str, keys: &[Key]) -> Result<(), ModelError> {
        let bad = |why: String| ModelError::Clip(name.into(), why);
        if name.is_empty() {
            return Err(bad("it needs a name".into()));
        }
        if self.clips.iter().any(|c| c.name == name) {
            return Err(bad("another clip has this name".into()));
        }
        if keys.is_empty() {
            return Err(bad("it needs a key".into()));
        }
        let mut last = -1.0;
        let before = self.pose_values();
        let mut check = || -> Result<(), ModelError> {
            for (time, pose) in keys {
                if !(time.is_finite() && *time >= 0.0 && *time > last) {
                    return Err(bad(format!(
                        "its times must be 0 or more and rise: {time} after {}",
                        if last < 0.0 { 0.0 } else { last }
                    )));
                }
                last = *time;
                if pose.is_empty() {
                    return Err(bad(format!("its key at {time} moves no joint")));
                }
                for (joint, value) in pose {
                    self.pose(joint, *value)?;
                }
            }
            Ok(())
        };
        let checked = check();
        // The checks posed the joints; put them back.
        for (joint, value) in before {
            let k = self.joint_index(&joint)?;
            self.joints[k].value = value;
        }
        checked?;
        self.clips.push(Clip {
            name: name.into(),
            keys: keys.to_vec(),
        });
        Ok(())
    }

    /// The clips' names, in declaration order.
    pub fn clips(&self) -> Vec<&str> {
        self.clips.iter().map(|c| c.name.as_str()).collect()
    }

    /// Poses the model as clip `name` has it at `time` seconds: each
    /// joint the clip moves, between the keys that name it (linear), and
    /// at its first or last value outside them. Joints the clip does not
    /// move keep their values.
    pub fn pose_at(&mut self, name: &str, time: f64) -> Result<(), ModelError> {
        if !time.is_finite() {
            return Err(ModelError::BadValue(format!("clip {name:?}'s time")));
        }
        let Some(clip) = self.clips.iter().find(|c| c.name == name).cloned() else {
            return Err(ModelError::Clip(
                name.into(),
                format!("no such clip; clips: {}", self.clips().join(", ")),
            ));
        };
        let mut tracks: BTreeMap<&str, Vec<(f64, f64)>> = BTreeMap::new();
        for (t, pose) in &clip.keys {
            for (joint, value) in pose {
                tracks.entry(joint.as_str()).or_default().push((*t, *value));
            }
        }
        for (joint, track) in tracks {
            let after = track.partition_point(|&(t, _)| t <= time);
            let value = if after == 0 {
                track[0].1
            } else if after == track.len() {
                track[after - 1].1
            } else {
                let ((t0, v0), (t1, v1)) = (track[after - 1], track[after]);
                v0 + (v1 - v0) * ((time - t0) / (t1 - t0))
            };
            self.pose(joint, value)?;
        }
        Ok(())
    }

    /// The first joint (nearest the node, then up through its parents)
    /// with a non-zero value that moves node `id`.
    pub(crate) fn moved_by(&self, id: Id) -> Option<usize> {
        let mut seen = BTreeSet::new();
        let mut at = self.body(id);
        while let Some(b) = at {
            if !seen.insert(b) {
                break;
            }
            let chain: Vec<usize> = (0..self.joints.len())
                .filter(|&k| self.joints[k].child == b)
                .collect();
            if let Some(&k) = chain.iter().rev().find(|&&k| self.value_of(k) != 0.0) {
                return Some(k);
            }
            at = chain
                .first()
                .and_then(|&k| self.body(self.joints[k].parent));
        }
        None
    }

    /// Refuses a read (`what`) of nodes the pose moves: every frame the
    /// model gives is at rest.
    pub(crate) fn at_rest(&self, what: &str, ids: &[Id]) -> Result<(), ModelError> {
        for &id in ids {
            for i in self.subtree(id) {
                if let Some(k) = self.moved_by(i) {
                    return Err(ModelError::Posed {
                        read: what.into(),
                        part: self.nodes[i.0 as usize].name.clone(),
                        joint: self.joints[k].desc.name.clone(),
                        value: self.value_of(k),
                    });
                }
            }
        }
        Ok(())
    }

    /// How each joint holds, at rest, in declaration order ([`Seating`]):
    /// the pivot's distance to the parent's and the child's surfaces, and
    /// how far the parent has moved off the pivot since the joint was
    /// declared. A joint frame read on another part than the parent, or a
    /// parent moved after, shows here; nothing is refused.
    pub fn seating(&self) -> Result<Vec<Seating>, ModelError> {
        let nearest = |ids: &[Id], p: DVec3| -> Result<Option<f64>, ModelError> {
            let solids: Vec<Solid> = ids
                .iter()
                .map(|&i| self.world_solid(i))
                .collect::<Result<_, _>>()?;
            let refs: Vec<&Solid> = solids.iter().collect();
            Ok(place::snap(&refs, &[p], None)
                .first()
                .map(|f| (f.origin - p).length()))
        };
        let live = |root: Id, not: Option<Id>| -> Vec<Id> {
            self.subtree(root)
                .into_iter()
                .filter(|&i| {
                    let n = &self.nodes[i.0 as usize];
                    !n.used_up && n.solid.is_some() && not.is_none_or(|c| !self.under(i, c))
                })
                .collect()
        };
        let mut out = Vec::with_capacity(self.joints.len());
        for (k, j) in self.joints.iter().enumerate() {
            let now = j
                .desc
                .on
                .inverse()
                .compose(&self.nodes[j.parent.0 as usize].placement);
            // The turn between the two: ‖R − I‖ = 2√2 sin(θ/2), exact
            // enough near 0 where acos is not.
            let d = [now.x - j.seat.x, now.y - j.seat.y, now.z - j.seat.z];
            let norm = d.iter().map(|v| v.length_squared()).sum::<f64>().sqrt();
            let half = (norm / (2.0 * 2.0f64.sqrt())).min(1.0);
            out.push(Seating {
                joint: j.desc.name.clone(),
                child: self.nodes[j.child.0 as usize].name.clone(),
                parent: self.nodes[j.parent.0 as usize].name.clone(),
                value: self.value_of(k),
                pivot_to_parent: nearest(&live(j.parent, Some(j.child)), j.desc.on.origin)?,
                pivot_to_child: nearest(&live(j.child, None), j.desc.on.origin)?,
                parent_moved: (now.origin - j.seat.origin).length(),
                parent_turned: 2.0 * og_core::math::double::asin(half),
            });
        }
        Ok(out)
    }
}
