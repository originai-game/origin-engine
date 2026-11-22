//! The tree's parts, groups and assemblies: adding, naming, grouping, placing, copying, mirroring and deleting them, and the frames read off them.

use super::*;

impl Model {
    pub(crate) fn node(&self, id: Id) -> Result<&Node, ModelError> {
        let n = self
            .nodes
            .get(id.0 as usize)
            .ok_or(ModelError::UnknownId(id.0))?;
        if n.deleted {
            return Err(ModelError::Deleted(n.name.clone()));
        }
        if n.used_up {
            return Err(ModelError::UsedUp(n.name.clone()));
        }
        Ok(n)
    }

    pub(crate) fn solid_of(&self, id: Id) -> Result<&Solid, ModelError> {
        let n = self.node(id)?;
        n.solid
            .as_ref()
            .ok_or_else(|| ModelError::NotAPart(n.name.clone()))
    }

    /// The node's name.
    pub fn name_of(&self, id: Id) -> Result<&str, ModelError> {
        Ok(&self.node(id)?.name)
    }

    /// The name of node `id`, used up or not; `None` for no such node.
    pub(crate) fn label(&self, id: Id) -> Option<&str> {
        self.nodes.get(id.0 as usize).map(|n| n.name.as_str())
    }

    /// The live node named `name`.
    pub fn find(&self, name: &str) -> Option<Id> {
        self.nodes
            .iter()
            .position(|n| !n.used_up && n.name == name)
            .map(|i| Id(i as u32))
    }

    pub(crate) fn check_name(&self, name: &str) -> Result<(), ModelError> {
        if name.is_empty() {
            return Err(ModelError::EmptyName);
        }
        if self.find(name).is_some() {
            return Err(ModelError::NameTaken(name.into()));
        }
        Ok(())
    }

    /// `name` if free, else `name.001`, `name.002`… (Blender's suffix, as a
    /// copy gets in sand3d).
    pub(crate) fn unique(&self, name: &str) -> String {
        let mut out = name.to_string();
        let mut k = 0;
        while self.find(&out).is_some() {
            k += 1;
            out = format!("{name}.{k:03}");
        }
        out
    }

    pub(crate) fn push(&mut self, node: Node) -> Id {
        self.nodes.push(node);
        Id(self.nodes.len() as u32 - 1)
    }

    /// A part named `name` holding `solid`, its own space on the world's.
    pub fn add(&mut self, name: &str, solid: Solid) -> Result<Id, ModelError> {
        self.check_name(name)?;
        if solid.positions.iter().any(|p| !p.is_finite()) {
            return Err(ModelError::BadValue(format!("{name:?}'s positions")));
        }
        Ok(self.push(Node {
            name: name.into(),
            solid: Some(solid),
            placement: Frame::IDENTITY,
            parent: None,
            material: None,
            anchors: BTreeMap::new(),
            used_up: false,
            deleted: false,
        }))
    }

    /// Whether `inner` is `outer` or somewhere under it.
    pub(crate) fn under(&self, inner: Id, outer: Id) -> bool {
        let mut at = Some(inner);
        while let Some(i) = at {
            if i == outer {
                return true;
            }
            at = self.nodes[i.0 as usize].parent;
        }
        false
    }

    /// The live nodes under `id`, itself first, in id order below it.
    pub(crate) fn subtree(&self, id: Id) -> Vec<Id> {
        let mut out = vec![id];
        for i in 0..self.nodes.len() {
            let i = Id(i as u32);
            if i != id && !self.nodes[i.0 as usize].used_up && self.under(i, id) {
                out.push(i);
            }
        }
        out
    }

    /// A group named `name` holding `children`, which keep their places.
    /// A child already in a group moves to this one.
    pub fn group(&mut self, name: &str, children: &[Id]) -> Result<Id, ModelError> {
        self.check_name(name)?;
        for &c in children {
            self.node(c)?;
        }
        for (k, &a) in children.iter().enumerate() {
            for &b in &children[k + 1..] {
                if self.under(a, b) || self.under(b, a) {
                    return Err(ModelError::Refused(
                        self.nodes[a.0 as usize].name.clone(),
                        "a group holds a node and one of its own parts".into(),
                    ));
                }
            }
        }
        let id = self.push(Node {
            name: name.into(),
            solid: None,
            placement: Frame::IDENTITY,
            parent: None,
            material: None,
            anchors: BTreeMap::new(),
            used_up: false,
            deleted: false,
        });
        for &c in children {
            self.nodes[c.0 as usize].parent = Some(id);
        }
        Ok(id)
    }

    /// An assembly: a group that publishes named frames (`anchors`, in
    /// the world) for details and joints to use.
    pub fn assembly(
        &mut self,
        name: &str,
        children: &[Id],
        anchors: &[(String, Frame)],
    ) -> Result<Id, ModelError> {
        for (n, f) in anchors {
            finite_frame(f, &format!("anchor {n:?}"))?;
        }
        let id = self.group(name, children)?;
        self.nodes[id.0 as usize].anchors = anchors.iter().cloned().collect();
        Ok(id)
    }

    /// The anchor `name` of an assembly, where the assembly is now.
    pub fn anchor(&self, assembly: Id, name: &str) -> Result<Frame, ModelError> {
        let n = self.node(assembly)?;
        self.at_rest("anchor", &[assembly])?;
        let local = n.anchors.get(name).ok_or_else(|| {
            ModelError::Refused(
                n.name.clone(),
                format!(
                    "no anchor named {name:?}; anchors: {}",
                    n.anchors.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            )
        })?;
        Ok(n.placement.compose(local))
    }

    /// A detail: `children` grouped under `name`, the group put under
    /// `assembly`.
    pub fn attach(&mut self, assembly: Id, name: &str, children: &[Id]) -> Result<Id, ModelError> {
        self.node(assembly)?;
        for &c in children {
            if self.under(assembly, c) {
                return Err(ModelError::Refused(
                    self.nodes[c.0 as usize].name.clone(),
                    "a detail cannot hold the assembly it is attached to".into(),
                ));
            }
        }
        let id = self.group(name, children)?;
        self.nodes[id.0 as usize].parent = Some(assembly);
        Ok(id)
    }

    /// Moves `id` and everything under it by the rigid motion `m` (world),
    /// with the pivots and axes of joints whose child is among them.
    pub(crate) fn move_by(&mut self, id: Id, m: &Frame) {
        let moved = self.subtree(id);
        for &i in &moved {
            let n = &mut self.nodes[i.0 as usize];
            n.placement = m.compose(&n.placement);
        }
        for j in &mut self.joints {
            if moved.contains(&j.child) {
                j.desc.on = m.compose(&j.desc.on);
                j.axis = j.axis.map(|a| m.vector_to_world(a));
                if j.desc.axis != DVec3::ZERO {
                    j.desc.axis = m.vector_to_world(j.desc.axis);
                }
            }
        }
    }

    /// Puts `id`'s own space on `frame` (world); what is under it moves
    /// with it. Use [`Frame::translate`], [`Frame::turn`] and
    /// [`Frame::offset`] on the frame for offsets.
    pub fn at(&mut self, id: Id, frame: &Frame) -> Result<(), ModelError> {
        finite_frame(frame, "at")?;
        let m = motion(&self.node(id)?.placement, frame);
        self.move_by(id, &m);
        Ok(())
    }

    /// Moves `id` (and what is under it) by `motion`, a rigid motion in
    /// the world: turned about the world origin, then moved.
    pub fn transform(&mut self, id: Id, motion: &Frame) -> Result<(), ModelError> {
        finite_frame(motion, "transform")?;
        self.node(id)?;
        self.move_by(id, motion);
        Ok(())
    }

    /// `id`'s placement: its own space in the world.
    pub fn placement(&self, id: Id) -> Result<Frame, ModelError> {
        let p = self.node(id)?.placement;
        self.at_rest("placement", &[id])?;
        Ok(p)
    }

    /// A frame given in `id`'s own space (where it was built), in the
    /// world: a source's joint origin, as written.
    pub fn local(&self, id: Id, frame: &Frame) -> Result<Frame, ModelError> {
        let p = self.node(id)?.placement;
        self.at_rest("local", &[id])?;
        Ok(p.compose(frame))
    }

    /// A part's solid in the world, at rest (whatever the pose).
    pub fn world_solid(&self, id: Id) -> Result<Solid, ModelError> {
        let n = self.node(id)?;
        let s = self.solid_of(id)?.clone();
        Ok(placed(s, &n.placement))
    }

    /// A new part, the convex hull of the parts under `ids` as they stand
    /// in the world, at rest (`crate::generators::hull::convex_hull`, exact: the
    /// smallest convex solid holding them). The parts stay as they are.
    /// Its faces and edges are named `hull`. Refused when the parts' points
    /// all lie in one plane.
    pub fn wrap(&mut self, name: &str, ids: &[Id]) -> Result<Id, ModelError> {
        self.check_name(name)?;
        let points = self.points(ids)?;
        let hull = crate::generators::hull::convex_hull(&points)?;
        self.add(name, hull)
    }

    /// Every world position of the parts under `ids`.
    pub(crate) fn points(&self, ids: &[Id]) -> Result<Vec<DVec3>, ModelError> {
        let mut out = Vec::new();
        for &id in ids {
            self.node(id)?;
            for i in self.subtree(id) {
                let n = &self.nodes[i.0 as usize];
                if let Some(s) = &n.solid {
                    out.extend(s.positions.iter().map(|&p| n.placement.point_to_world(p)));
                }
            }
        }
        Ok(out)
    }

    /// The frame on the box bounding `ids` (and everything under them):
    /// `"+X"` a side's centre, `"+X-Y"` an edge's middle, `"+X-Y+Z"` a
    /// corner; its normal looks the first way named.
    pub fn bbox(&self, ids: &[Id], which: &str) -> Result<Frame, ModelError> {
        let points = self.points(ids)?;
        self.at_rest("bbox", ids)?;
        place::bbox(&points, which)
    }

    /// The frame on part `id`'s face named `which` (or furthest along a
    /// direction, `"+X"`): origin at the face's centroid, +Z along its
    /// normal ([`place::face`]).
    pub fn face(&self, id: Id, which: &str) -> Result<Frame, ModelError> {
        let s = self.world_solid(id)?;
        self.at_rest("face", &[id])?;
        place::face(&s, &self.nodes[id.0 as usize].name, which)
    }

    /// The edges of part `id` named `which`, as a path in the world
    /// ([`place::edge`]).
    pub fn edge(&self, id: Id, which: &str) -> Result<Path, ModelError> {
        let s = self.world_solid(id)?;
        self.at_rest("edge", &[id])?;
        place::edge(&s, &self.nodes[id.0 as usize].name, which)
    }

    /// Frames on the surface of the parts `targets` ([`place::snap`]).
    pub fn snap(
        &self,
        targets: &[Id],
        points: &[DVec3],
        direction: Option<DVec3>,
    ) -> Result<Vec<Frame>, ModelError> {
        let solids: Vec<Solid> = targets
            .iter()
            .map(|&t| self.world_solid(t))
            .collect::<Result<_, _>>()?;
        self.at_rest("snap", targets)?;
        let refs: Vec<&Solid> = solids.iter().collect();
        Ok(place::snap(&refs, points, direction))
    }

    /// The face and edge names part `id` carries, sorted.
    pub fn names(&self, id: Id) -> Result<(Vec<String>, Vec<String>), ModelError> {
        let s = self.solid_of(id)?;
        let faces: Vec<String> = s
            .face_tags
            .iter()
            .flat_map(|t| t.iter())
            .map(|&t| s.names.name(t).to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let edges: Vec<String> = s
            .edge_tags
            .values()
            .flatten()
            .map(|&t| s.names.name(t).to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok((faces, edges))
    }

    /// Copies `id` and everything under it with the geometry each node
    /// gets from `geometry`. The copy's root takes `name`, the nodes under
    /// it their own names with a free suffix. Answers the copy's root and
    /// each copied node's id by the original's.
    pub(crate) fn duplicate(
        &mut self,
        id: Id,
        name: &str,
        geometry: impl Fn(&Node) -> (Option<Solid>, Frame),
    ) -> Result<(Id, BTreeMap<Id, Id>), ModelError> {
        self.node(id)?;
        self.check_name(name)?;
        let tree = self.subtree(id);
        let mut map: BTreeMap<Id, Id> = BTreeMap::new();
        for &old in &tree {
            let src = self.nodes[old.0 as usize].clone();
            let (solid, placement) = geometry(&src);
            let new_name = if old == id {
                name.to_string()
            } else {
                self.unique(&src.name)
            };
            let parent = if old == id {
                src.parent
            } else {
                src.parent.and_then(|p| map.get(&p).copied())
            };
            let new = self.push(Node {
                name: new_name,
                solid,
                placement,
                parent,
                material: src.material.clone(),
                anchors: src.anchors.clone(),
                used_up: false,
                deleted: false,
            });
            map.insert(old, new);
        }
        Ok((map[&id], map))
    }

    /// `name` if no joint has it, else `name.001`, `name.002`…
    pub(crate) fn unique_joint(&self, name: &str) -> String {
        let taken = |n: &str| self.joints.iter().any(|j| j.desc.name == n);
        let mut out = name.to_string();
        let mut k = 0;
        while taken(&out) {
            k += 1;
            out = format!("{name}.{k:03}");
        }
        out
    }

    /// Copies every joint whose child was copied (`map`: original id to
    /// copy), in declaration order, onto the copies: each named with a
    /// free suffix (`hinge.001`), hanging on the parent's copy when the
    /// parent was copied too and on the same parent otherwise, following
    /// its leader's copy when that was copied. `image` maps the pivot, the
    /// axis (world) and the explicit axis of a joint of kind `kind`.
    pub(crate) fn copy_joints(
        &mut self,
        map: &BTreeMap<Id, Id>,
        image: impl Fn(JointKind, &Frame, Option<DVec3>, DVec3) -> (Frame, Option<DVec3>, DVec3),
    ) {
        let mut names: BTreeMap<String, String> = BTreeMap::new();
        let copied: Vec<JointRec> = self
            .joints
            .iter()
            .filter(|j| map.contains_key(&j.child))
            .cloned()
            .collect();
        for j in copied {
            let name = self.unique_joint(&j.desc.name);
            names.insert(j.desc.name.clone(), name.clone());
            let parent = map.get(&j.parent).copied().unwrap_or(j.parent);
            let (on, axis, explicit) = image(j.desc.kind, &j.desc.on, j.axis, j.desc.axis);
            let mimic = j.desc.mimic.as_ref().map(|m| Mimic {
                joint: names.get(&m.joint).cloned().unwrap_or(m.joint.clone()),
                multiplier: m.multiplier,
            });
            let seat = on
                .inverse()
                .compose(&self.nodes[parent.0 as usize].placement);
            self.joints.push(JointRec {
                child: map[&j.child],
                parent,
                desc: JointDesc {
                    name,
                    on,
                    axis: explicit,
                    mimic,
                    ..j.desc
                },
                axis,
                value: j.value,
                seat,
            });
        }
    }

    /// A copy of `id` and everything under it, in the same place, named
    /// `name` (the nodes under it keep their names with a suffix:
    /// `.001`…). The joints of the parts and groups copied come with them
    /// at the same pose, named with a free suffix (`hinge.001`): a copied
    /// part hangs on its parent's copy when the parent was copied too, on
    /// the same parent otherwise; a follower follows its leader's copy
    /// when that was copied.
    pub fn copy(&mut self, id: Id, name: &str) -> Result<Id, ModelError> {
        let (root, map) = self.duplicate(id, name, |n| (n.solid.clone(), n.placement))?;
        self.copy_joints(&map, |_, on, axis, explicit| (*on, axis, explicit));
        Ok(root)
    }

    /// The mirror image of `id` and everything under it across `plane`'s
    /// XY plane (its normal is +Z), named `name`. Each part's image is
    /// built into its solid (a mirror is no placement), so the copy's
    /// parts sit with their own space on the world's.
    ///
    /// Joints come as [`Model::copy`] brings them, each moving as the
    /// image of the original's motion (sand3d: "the image of the
    /// motion"): the pivot's origin mirrored, its x and y mirrored and its
    /// z their cross product. A slide's axis is the axis's mirror image; a
    /// turn's is the image negated (a mirror reverses the sense of a
    /// turn). Ranges, values and mimics are kept, so the same value poses
    /// the copy as the image of the original at that value.
    pub fn mirror(&mut self, id: Id, plane: &Frame, name: &str) -> Result<Id, ModelError> {
        finite_frame(plane, "mirror")?;
        let (o, n) = (plane.origin, plane.z);
        let image = move |p: DVec3| p - n * (2.0 * (p - o).dot(n));
        let turn = move |v: DVec3| v - n * (2.0 * v.dot(n));
        let (root, map) = self.duplicate(id, name, |node| {
            let solid = node.solid.clone().map(|mut s| {
                let f = node.placement;
                s.map_positions(|p| image(f.point_to_world(p)), true);
                rounded(s)
            });
            (solid, Frame::IDENTITY)
        })?;
        self.copy_joints(&map, |kind, on, axis, explicit| {
            let x = turn(on.x);
            let y = turn(on.y);
            let pivot = Frame {
                origin: image(on.origin),
                x,
                y,
                z: x.cross(y),
            };
            // A turn's axis is a pseudovector: its image is negated. A
            // slide's is a plain direction; its default (the pivot's z)
            // is written out, since the new pivot's z is the negated one.
            let slide = kind == JointKind::Prismatic;
            let image_of = |v: DVec3| if slide { turn(v) } else { -turn(v) };
            let explicit = if explicit != DVec3::ZERO {
                image_of(explicit)
            } else if slide {
                turn(on.z)
            } else {
                DVec3::ZERO
            };
            (pivot, axis.map(image_of), explicit)
        });
        Ok(root)
    }

    /// Removes `id` and everything under it. Joints whose child is
    /// removed go with it. Refused, changing nothing: a joint that hangs a
    /// node that stays on a node removed, or one that follows (`mimic`) a
    /// joint removed.
    pub fn delete(&mut self, id: Id) -> Result<(), ModelError> {
        let name = self.node(id)?.name.clone();
        let gone: BTreeSet<Id> = self.subtree(id).into_iter().collect();
        let dropped: BTreeSet<String> = self
            .joints
            .iter()
            .filter(|j| gone.contains(&j.child))
            .map(|j| j.desc.name.clone())
            .collect();
        for j in self.joints.iter().filter(|j| !gone.contains(&j.child)) {
            if gone.contains(&j.parent) {
                return Err(ModelError::Refused(
                    name,
                    format!(
                        "joint {:?} hangs {:?} on {:?}; delete {:?} too",
                        j.desc.name,
                        self.nodes[j.child.0 as usize].name,
                        self.nodes[j.parent.0 as usize].name,
                        self.nodes[j.child.0 as usize].name,
                    ),
                ));
            }
            if let Some(m) = j.desc.mimic.as_ref().filter(|m| dropped.contains(&m.joint)) {
                return Err(ModelError::Refused(
                    name,
                    format!(
                        "joint {:?} follows joint {:?}, which would go",
                        j.desc.name, m.joint
                    ),
                ));
            }
        }
        self.joints.retain(|j| !gone.contains(&j.child));
        for i in gone {
            let n = &mut self.nodes[i.0 as usize];
            n.used_up = true;
            n.deleted = true;
        }
        Ok(())
    }

    /// Scales part `id`'s solid along its own axes, about its own origin,
    /// by `factors` (each positive; a mirror is [`Model::mirror`]). Its
    /// placement, names and joints stay; a group is refused.
    pub fn scale(&mut self, id: Id, factors: DVec3) -> Result<(), ModelError> {
        if !(factors.is_finite() && factors.min_element() > 0.0) {
            return Err(ModelError::BadValue(
                "a scale factor (each must be positive)".into(),
            ));
        }
        self.edit(id, |s, _| {
            let mut s = s.clone();
            s.map_positions(|p| p * factors, false);
            s.canonicalize()
                .map_err(|_| ModelError::BadValue("the scaled positions".into()))?;
            Ok(s)
        })
    }

    /// Every live part in the world at rest, in id order.
    pub(crate) fn parts(&self) -> Vec<Id> {
        (0..self.nodes.len() as u32)
            .map(Id)
            .filter(|&i| {
                let n = &self.nodes[i.0 as usize];
                !n.used_up && n.solid.is_some()
            })
            .collect()
    }

    /// The scene part a joint hanging on node `p` hangs on: a part's own,
    /// a jointed group's link, or the body it moves with ("" for the
    /// root).
    pub(crate) fn representative(&self, p: Id) -> String {
        let n = &self.nodes[p.0 as usize];
        if n.solid.is_some() || self.joints.iter().any(|j| j.child == p) {
            return n.name.clone();
        }
        match self.body(p) {
            Some(b) => self.nodes[b.0 as usize].name.clone(),
            None => String::new(),
        }
    }
}
