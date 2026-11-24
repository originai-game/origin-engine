use super::*;

/// Whether a collider's shape is built from a registered asset (a mesh or a
/// height field), so a save carries its key instead of its geometry.
fn from_asset(c: &Collider) -> bool {
    matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
        && c.mesh & crate::GENERATED_MESH == 0
}

fn asset_key(c: &Collider, scale: Vec3) -> ShapeKey {
    let s = if c.shape == SHAPE_HEIGHTFIELD {
        scale.abs()
    } else {
        Vec3::new(
            if scale.x == 0.0 { 0.0 } else { scale.x },
            if scale.y == 0.0 { 0.0 } else { scale.y },
            if scale.z == 0.0 { 0.0 } else { scale.z },
        )
    };
    ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: s.to_array().map(f32::to_bits),
    }
}

/// `sim`'s bytes without the geometry of its mesh, convex-hull and
/// height-field colliders: meshes are assets, so a save carries each such
/// shape's key and [`load`] rebuilds it from the asset. Everything else is
/// rapier's own serialisation, byte for byte, written in a single pass.
pub(crate) fn save(sim: &Sim) -> (Vec<Hole>, Vec<u8>) {
    let w = &sim.world;
    // Which colliders' shapes come from assets, by handle.
    let mut keyed: BTreeMap<(u32, u32), ShapeKey> = BTreeMap::new();
    for (key, t) in sim.bodies.iter() {
        let Some(rb) = w.bodies.get(t.handle) else {
            continue;
        };
        if let Some(c) = t.collider.as_ref().filter(|c| from_asset(c))
            && let Some(&h) = rb.colliders().first()
        {
            keyed.insert(h.into_raw_parts(), asset_key(c, t.scale));
        }
        // A compound's children follow its own collider, in build order.
        if let Some(children) = sim.ext.compounds.get(key) {
            let skip = usize::from(t.collider.is_some());
            for (child, &h) in children.iter().zip(rb.colliders().iter().skip(skip)) {
                if from_asset(&child.collider) {
                    keyed.insert(h.into_raw_parts(), asset_key(&child.collider, child.scale));
                }
            }
        }
    }
    for (key, t) in sim.statics.iter() {
        if !from_asset(&t.collider) {
            continue;
        }
        let whole = asset_key(&t.collider, t.scale);
        match sim.ext.fields.get(key) {
            Some(chunks) => {
                for (k, &h) in chunks.iter().enumerate() {
                    keyed.insert(h.into_raw_parts(), crate::chunk_key(whole, k as u32));
                }
            }
            None => {
                keyed.insert(t.handle.into_raw_parts(), whole);
            }
        }
    }
    let slot_keys: Vec<Option<ShapeKey>> = if keyed.is_empty() {
        Vec::new()
    } else {
        w.colliders
            .iter()
            .map(|(h, _)| keyed.get(&h.into_raw_parts()).copied())
            .collect()
    };
    crate::encode_stripped(sim, &slot_keys)
}

/// The simulation [`save`] wrote, its asset shapes rebuilt from the meshes
/// and height fields registered now (the same shapes a step builds, from
/// the shape cache when it has them). Refused, naming the handle, for a
/// mesh no longer registered.
pub(crate) fn load(bytes: &[u8], holes: &[Hole], shared: &mut Shared) -> Result<Sim> {
    use bincode::Options;
    let parse = |e: bincode::Error| {
        Error::new(
            ErrorKind::Parse,
            format!("the rapier world does not decode: {e}"),
        )
    };
    let codec = crate::codec;
    if holes.is_empty() {
        return codec().deserialize(bytes).map_err(parse);
    }
    let mut full = Vec::with_capacity(bytes.len());
    let mut from = 0;
    for hole in holes {
        let at = usize::try_from(hole.at)
            .ok()
            .filter(|&at| at >= from && at <= bytes.len())
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Parse,
                    format!(
                        "the physics block's asset shape offset {} is out of order or past {}",
                        hole.at,
                        bytes.len()
                    ),
                )
            })?;
        full.extend_from_slice(&bytes[from..at]);
        from = at;
        let c = Collider {
            shape: hole.key.shape,
            mesh: hole.key.mesh,
            ..Collider::default()
        };
        let scale = Vec3::from_array(hole.key.scale.map(f32::from_bits));
        let built = shape(&c, scale, shared).map_err(|e| {
            e.context(
                "the save's colliders are built from meshes and height fields, which are assets: load or register them before restoring",
            )
        })?;
        crate::encode(&mut full, &built).map_err(parse)?;
    }
    full.extend_from_slice(&bytes[from..]);
    codec().deserialize(&full).map_err(parse)
}

use og_core::block_form::{Field, Record, Records};

fn named(list: Vec<(&str, Field)>) -> Vec<(String, Field)> {
    list.into_iter().map(|(k, f)| (k.to_owned(), f)).collect()
}

/// The entity a rapier body or collider was built for (its user data's low
/// bits; 0 for the ground body).
pub(crate) fn owner(user_data: u128) -> Field {
    Field::Entity(user_data as u64)
}

/// A rapier body's simulated state and settings by name (`before`: the z
/// a 2D pose keeps), no handle among them.
pub(crate) fn body_fields(rb: &rp::RigidBody, before: Vec3) -> Vec<(String, Field)> {
    let (position, rotation) = unpose(rb.position(), before);
    let (next_position, next_rotation) = unpose(rb.next_position(), before);
    named(vec![
        ("type", Field::of(&format!("{:?}", rb.body_type()))),
        ("position", Field::of(&position)),
        ("rotation", Field::of(&rotation)),
        ("linear", Field::of(&unv(rb.linvel()))),
        ("angular", Field::of(&unangvel(rb.angvel()))),
        ("sleeping", Field::of(&rb.is_sleeping())),
        ("nextPosition", Field::of(&next_position)),
        ("nextRotation", Field::of(&next_rotation)),
        ("mass", Field::of(&rb.mass())),
        ("centerOfMass", Field::of(&unv(rb.center_of_mass()))),
        ("linearDamping", Field::of(&rb.linear_damping())),
        ("angularDamping", Field::of(&rb.angular_damping())),
        ("gravityScale", Field::of(&rb.gravity_scale())),
        ("lock", Field::of(&rb.locked_axes().bits())),
        ("ccd", Field::of(&rb.is_ccd_enabled())),
        ("dominance", Field::of(&rb.dominance_group())),
        ("force", Field::of(&unv(rb.user_force()))),
        ("torque", Field::of(&unangvel(rb.user_torque()))),
        ("enabled", Field::of(&rb.is_enabled())),
        ("colliders", Field::of(&rb.colliders().len())),
    ])
}

/// A rapier collider's shape and settings by name, its owner as an entity.
pub(crate) fn collider_fields(c: &rp::Collider) -> Vec<(String, Field)> {
    let (position, rotation) = unpose(c.position(), Vec3::ZERO);
    let (local_position, local_rotation) = c
        .position_wrt_parent()
        .map_or((Vec3::ZERO, Quat::IDENTITY), |p| unpose(p, Vec3::ZERO));
    let aabb = c.shape().compute_aabb(&rp::Pose::IDENTITY);
    let (groups, solver) = (c.collision_groups(), c.solver_groups());
    named(vec![
        ("entity", owner(c.user_data)),
        ("shape", Field::of(&format!("{:?}", c.shape().shape_type()))),
        ("bounds", Field::of(&(unv(aabb.mins), unv(aabb.maxs)))),
        ("position", Field::of(&position)),
        ("rotation", Field::of(&rotation)),
        ("localPosition", Field::of(&local_position)),
        ("localRotation", Field::of(&local_rotation)),
        ("friction", Field::of(&c.friction())),
        ("restitution", Field::of(&c.restitution())),
        ("density", Field::of(&c.density())),
        ("mass", Field::of(&c.mass())),
        ("sensor", Field::of(&c.is_sensor())),
        ("layers", Field::of(&groups.memberships.bits())),
        ("mask", Field::of(&groups.filter.bits())),
        ("solverLayers", Field::of(&solver.memberships.bits())),
        ("solverMask", Field::of(&solver.filter.bits())),
        ("events", Field::of(&c.active_events().bits())),
        ("hooks", Field::of(&c.active_hooks().bits())),
        ("skin", Field::of(&c.contact_skin())),
        ("enabled", Field::of(&c.is_enabled())),
    ])
}

/// A rapier joint's state by name: the entities of its two bodies in
/// rapier's order (`body1`, `body2`; `anchor1` and `anchor2` in their
/// frames), its description (frames, axes, limits, motors) and its
/// warm-start impulses.
pub(crate) fn joint_fields(
    j: &rp::ImpulseJoint,
    bodies: &rp::RigidBodySet,
) -> Vec<(String, Field)> {
    let of = |h: rp::RigidBodyHandle| {
        bodies
            .get(h)
            .map_or(Field::Entity(0), |b| owner(b.user_data))
    };
    named(vec![
        ("body1", of(j.body1())),
        ("body2", of(j.body2())),
        ("anchor1", Field::of(&unv(j.data.local_anchor1()))),
        ("anchor2", Field::of(&unv(j.data.local_anchor2()))),
        ("data", Field::of(&j.data)),
        ("impulses", Field::of(&j.impulses)),
    ])
}

/// One record per collider in `handles` (`kind`, keyed by `key` of its
/// index there).
pub(crate) fn collider_records(
    out: &mut Vec<Record>,
    entity: u64,
    kind: &str,
    handles: &[rp::ColliderHandle],
    w: &rp::PhysicsWorld,
    key: &dyn Fn(usize) -> Field,
) {
    for (i, &h) in handles.iter().enumerate() {
        if let Some(c) = w.colliders.get(h) {
            let mut r = Record::new(entity, kind).key(key(i));
            r.fields = collider_fields(c);
            out.push(r);
        }
    }
}

/// A body's record of `kind` (its rapier state), `missing` when its handle
/// names none.
pub(crate) fn body_record(
    entity: u64,
    kind: &str,
    key: Field,
    h: rp::RigidBodyHandle,
    w: &rp::PhysicsWorld,
    before: Vec3,
) -> Record {
    let mut r = Record::new(entity, kind).key(key);
    match w.bodies.get(h) {
        Some(rb) => r.fields = body_fields(rb, before),
        None => r = r.field("missing", Field::of(&true)),
    }
    r
}

/// Each `(entity, IdMap<u64, bool>)` side map as records keyed by the body.
fn sides(out: &mut Vec<Record>, entity: u64, kind: &str, map: &IdMap<u64, bool>) {
    for (&body, &under) in map.iter() {
        out.push(
            Record::new(entity, kind)
                .key(Field::Entity(body))
                .field("under", Field::of(&under)),
        );
    }
}

/// `value`'s members without `skip` (a map keyed by entity id, given as
/// records of its own).
fn members_but(r: Record, value: &impl Serialize, skip: &str) -> Record {
    let mut r = r.members(value, &[]);
    r.fields.retain(|(k, _)| k != skip);
    r
}

impl Sim {
    /// The simulation by entity ([`og_core::block_form`]): every body,
    /// collider, joint, vehicle, ragdoll, rope, cloth, fluid, soft volume,
    /// garment and water state of each entity, the touching contact pairs
    /// on their first collider's entity, and in `out.global` what is no
    /// entity's. No rapier handle or arena index appears; references are
    /// by entity.
    pub(crate) fn form(&self, out: &mut Records) {
        let w = &self.world;
        out.global.extend(named(vec![
            ("gravity", Field::of(&unv(w.gravity))),
            ("integration", Field::of(&w.integration_parameters)),
            ("ground", Field::of(&self.ground.is_some())),
            ("hooks", Field::of(&self.ext.hooked)),
        ]));
        let r = &mut out.entities;
        for (&e, t) in self.bodies.iter() {
            let mut rec = body_record(e, "body", Field::of(&()), t.handle, w, t.position)
                .field("last.body", Field::of(&t.body))
                .field("last.collider", Field::of(&t.collider))
                .field("last.scale", Field::of(&t.scale))
                .field("last.position", Field::of(&t.position))
                .field("last.quaternion", Field::of(&t.quaternion))
                .field("last.velocity", Field::of(&t.velocity))
                .field(
                    "still",
                    Field::of(&self.ext.still.get(&e).copied().unwrap_or(0)),
                );
            let children = self.ext.compounds.get(&e);
            if let Some(children) = children {
                rec = rec.field(
                    "children",
                    Field::Entities(children.iter().map(|c| c.entity).collect()),
                );
            }
            r.push(rec);
            let Some(rb) = w.bodies.get(t.handle) else {
                continue;
            };
            let first = r.len();
            collider_records(r, e, "collider", rb.colliders(), w, &|i| Field::of(&i));
            // A compound's children follow its own collider, in build order.
            if let Some(children) = children {
                let skip = usize::from(t.collider.is_some());
                for (rec, c) in r[first..].iter_mut().skip(skip).zip(children) {
                    rec.fields.push((
                        "last".to_owned(),
                        Field::of(&(c.position, c.quaternion, c.scale, c.collider)),
                    ));
                }
            }
        }
        for (&e, t) in self.statics.iter() {
            r.push(
                Record::new(e, "static")
                    .field("last.collider", Field::of(&t.collider))
                    .field("last.scale", Field::of(&t.scale))
                    .field("last.position", Field::of(&t.position))
                    .field("last.quaternion", Field::of(&t.quaternion)),
            );
            let handles = self
                .ext
                .fields
                .get(&e)
                .cloned()
                .unwrap_or_else(|| vec![t.handle]);
            collider_records(r, e, "collider", &handles, w, &|i| Field::of(&i));
        }
        ext::joint_records(&self.joints, w, r);
        for (&e, s) in self.vehicles.iter() {
            s.form(e, w, r);
            if let Some(x) = self.ext.vehicles.get(&e) {
                let mounts: Vec<[f32; 5]> = x
                    .wheels
                    .iter()
                    .map(|(_, b)| b.map(f32::from_bits))
                    .collect();
                r.push(
                    Record::new(e, "vehicle ext")
                        .field("gearbox", Field::of(&x.gearbox))
                        .field(
                            "wheels",
                            Field::Entities(x.wheels.iter().map(|w| w.0).collect()),
                        )
                        .field("mounts", Field::of(&mounts)),
                );
            }
        }
        for (&e, s) in self.ragdolls.iter() {
            s.form(e, w, r);
        }
        for (&e, s) in self.ropes.iter() {
            r.push(Record::new(e, "rope").members(s, &[]));
        }
        for (&e, s) in self.cloths.iter() {
            r.push(Record::new(e, "cloth").members(s, &[]));
        }
        for (&e, s) in self.fluids.iter() {
            r.push(Record::new(e, "fluid").members(s, &[]));
        }
        for (&e, s) in self.fluids3.iter() {
            r.push(Record::new(e, "fluid3d").members(s, &[]));
        }
        for (&e, s) in self.ext.softs.iter() {
            r.push(Record::new(e, "soft").members(s, &[]));
        }
        for (&e, s) in self.ext.garments.iter() {
            r.push(Record::new(e, "garment").members(s, &[]));
        }
        for (&e, s) in self.surfaces.iter() {
            r.push(members_but(Record::new(e, "surface"), s, "under"));
            sides(r, e, "surface under", &s.under);
        }
        for (&e, s) in self.lines.iter() {
            r.push(members_but(Record::new(e, "line"), s, "under"));
            sides(r, e, "line under", &s.under);
        }
        for (&e, map) in self.ext.under.iter() {
            sides(r, e, "water under", map);
        }
        contact_records(w, r);
    }
}

/// One contact point's place, the same in two worlds whatever handles
/// their colliders hold: on each side, the collider's index among its
/// body's colliders, the shape feature it touches and its local point's
/// bits, the two sides in sorted order (so which collider rapier names
/// first does not matter).
type PointKey = [(u32, u32, [u32; 3]); 2];

/// Sums each point's (normal impulse, warm-start impulse) in [`PointKey`]
/// order: two equal worlds sum the same numbers in the same order, so the
/// sums are equal to the bit, with no tolerance.
pub(crate) fn canonical_sums(points: &mut [(PointKey, f32, f32)]) -> (f32, f32) {
    points.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.to_bits().cmp(&b.1.to_bits()))
            .then(a.2.to_bits().cmp(&b.2.to_bits()))
    });
    points
        .iter()
        .fold((0.0, 0.0), |(i, w), p| (i + p.1, w + p.2))
}

/// The narrow phase by entity: per touching pair (summed over the pairs
/// between the same two entities' colliders), a `contact` record on the
/// first collider's entity keyed by the second's, and the same on the
/// second's keyed by the first's (which collider rapier names first follows
/// the handles, which differ between worlds); per intersecting sensor
/// pair, a `sensor contact` likewise. Impulses are summed in a canonical
/// order ([`canonical_sums`]), never in the narrow phase's handle order.
fn contact_records(w: &rp::PhysicsWorld, out: &mut Vec<Record>) {
    let entity = |h: rp::ColliderHandle| w.colliders.get(h).map_or(0, |c| c.user_data as u64);
    // A collider's index among its body's colliders (0 for one with no body).
    let index = |h: rp::ColliderHandle| {
        w.colliders
            .get(h)
            .and_then(|c| c.parent())
            .and_then(|b| w.bodies.get(b))
            .and_then(|b| b.colliders().iter().position(|&c| c == h))
            .unwrap_or(0) as u32
    };
    let bits = |p: &rp::Vector| {
        let a: [f32; 3] = {
            let a = p.to_array();
            let v: &[f32] = &a;
            [v[0], v[1], v.get(2).copied().unwrap_or(0.0)]
        };
        a.map(f32::to_bits)
    };
    /// Pairs, manifolds, points, and every point's key and impulses.
    type Sums = (u32, u32, u32, Vec<(PointKey, f32, f32)>);
    let mut touching: BTreeMap<(u64, u64), Sums> = BTreeMap::new();
    for p in w.narrow_phase.contact_pairs() {
        if !p.has_any_active_contact() {
            continue;
        }
        let (a, b) = (entity(p.collider1), entity(p.collider2));
        let (i1, i2) = (index(p.collider1), index(p.collider2));
        let s = touching.entry((a.min(b), a.max(b))).or_default();
        s.0 += 1;
        for m in p.solver_manifolds() {
            s.1 += 1;
            s.2 += m.points.len() as u32;
            for c in &m.points {
                let mut key = [
                    (i1, c.fid1.0, bits(&c.local_p1)),
                    (i2, c.fid2.0, bits(&c.local_p2)),
                ];
                key.sort_unstable();
                s.3.push((key, c.data.impulse, c.data.warmstart_impulse));
            }
        }
    }
    let both = |a: u64, b: u64| {
        if a == b {
            vec![(a, b)]
        } else {
            vec![(a, b), (b, a)]
        }
    };
    for ((a, b), (pairs, manifolds, points, mut each)) in touching {
        let (impulse, warm) = canonical_sums(&mut each);
        for (on, with) in both(a, b) {
            out.push(
                Record::new(on, "contact")
                    .key(Field::Entity(with))
                    .field("pairs", Field::of(&pairs))
                    .field("manifolds", Field::of(&manifolds))
                    .field("points", Field::of(&points))
                    .field("impulse", Field::of(&impulse))
                    .field("warmstart", Field::of(&warm)),
            );
        }
    }
    let mut sensed: BTreeMap<(u64, u64), u32> = BTreeMap::new();
    for (h1, h2, on) in w.narrow_phase.intersection_pairs() {
        if on {
            let (a, b) = (entity(h1), entity(h2));
            *sensed.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    for ((a, b), pairs) in sensed {
        for (on, with) in both(a, b) {
            out.push(
                Record::new(on, "sensor contact")
                    .key(Field::Entity(with))
                    .field("pairs", Field::of(&pairs)),
            );
        }
    }
}
