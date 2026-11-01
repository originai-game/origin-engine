//! Cached entity hierarchy and world transforms.

use super::*;

/// Local transforms and parents of every entity with `og.Transform`, for
/// composing world matrices.
///
/// Dense by entity index (an entity's slot holds its bits, so a stale or
/// unknown entity is told apart), read from the columns without decoding a
/// row: extraction asks it for every drawn entity on every frame, so a
/// lookup is an index, not a tree walk (owed row 50: 20,000 meshes spent
/// most of their frame in the maps this replaced).
#[derive(Default)]
pub struct Hierarchy {
    /// Per entity index: the entity's bits and what is known of its
    /// matrices, apart from them (`data`), so a read where nothing moved
    /// walks 16 bytes an entity, not the matrices' 190 (20,000 still
    /// meshes: 0.33 ms a read, read twice a frame).
    slots: Vec<Slot>,
    /// Per entity index: its transform's bytes, its local matrix and its
    /// world matrix once composed.
    data: Vec<SlotData>,
    /// Parents by child bits (only entities with `og.Parent`).
    parent: IdMap<u64, u64>,
    /// World matrices of chain members without `og.Transform` (no slot).
    loose: IdMap<u64, Mat4>,
    /// World matrices composed since the last read of the world (the
    /// raycast record's cost test counts them).
    pub(crate) composed: usize,
    /// Which slots had og.Transform at the last read (kept for its
    /// allocation).
    was: Vec<bool>,
    /// Per og.Parent archetype: its entities and its column as last read.
    parent_columns: Vec<(Vec<Entity>, Vec<u8>)>,
    /// Per og.Transform archetype, in query order: its entities and its
    /// position, quaternion and scale columns as last read, so an
    /// archetype where nothing moved is told by four comparisons.
    columns: Vec<(Vec<Entity>, [Vec<u8>; 3])>,
    /// A read of the world ([`Hierarchy::capture`]) not yet applied to the
    /// slots: they are brought to `columns` and `parent` when an entity is
    /// next asked for.
    pending: Option<Pending>,
    /// How many archetypes of `columns` the slots were last brought to.
    applied: usize,
    /// Entities and slots the last [`Hierarchy::apply`] walked: none when
    /// nothing moved since the one before (the raycast record's cost test
    /// counts them).
    pub(crate) walked: usize,
    /// A display frame between ticks: [`Hierarchy::world_of`] answers the
    /// blend of the previous tick's world matrix and this one's
    /// (`crate::interpolate`).
    blend: Option<Box<Blend>>,
}

/// What a display frame between two ticks blends with
/// ([`Hierarchy::set_blend`]).
pub(crate) struct Blend {
    /// The transforms as the last tick started.
    pub prev: Hierarchy,
    /// 0 draws `prev`, 1 the tick's own.
    pub alpha: f32,
    /// The squared distance past which a move between the ticks is a
    /// teleport, drawn at the tick (infinite: none).
    pub snap_sq: f32,
    /// Entities drawn at the tick with everything under them
    /// (og.Interpolation.off, og.render.snap), by bits.
    pub held: IdMap<u64, ()>,
}

impl Blend {
    /// The drawn world matrix of `e`, whose tick world matrix is `m`
    /// (`parent`: the tick's og.Parent links).
    fn of(&mut self, parent: &IdMap<u64, u64>, e: u64, m: Mat4) -> Mat4 {
        if !self.held.is_empty() {
            let mut cur = e;
            for _ in 0..=256 {
                if self.held.contains_key(&cur) {
                    return m;
                }
                match parent.get(&cur) {
                    Some(&p) => cur = p,
                    None => break,
                }
            }
        }
        // Spawned during the tick: nothing to come from.
        if !self.prev.knows(e) {
            return m;
        }
        blend(self.prev.world_of(e), m, self.alpha, self.snap_sq)
    }
}

/// `a` to `b` at `t`: translation and scale lerped, rotation slerped with
/// og-core's math (`og_core::math::quat_slerp`, no host `libm`); `b` when
/// the translation moved more than `snap_sq`'s root, and either end exactly
/// at 0 and 1.
fn blend(a: Mat4, b: Mat4, t: f32, snap_sq: f32) -> Mat4 {
    if a == b || t >= 1.0 {
        return b;
    }
    if t <= 0.0 {
        return a;
    }
    let (s0, r0, t0) = a.to_scale_rotation_translation();
    let (s1, r1, t1) = b.to_scale_rotation_translation();
    if t0.distance_squared(t1) > snap_sq {
        return b;
    }
    let r = if r0 == r1 {
        r1
    } else {
        og_core::math::quat_slerp(r0, r1, t).normalize()
    };
    Mat4::from_scale_rotation_translation(s0.lerp(s1, t), r, t0.lerp(t1, t))
}

/// What [`Hierarchy::capture`] leaves for [`Hierarchy::apply`]: per
/// archetype of `columns`, whether its entities and columns are those the
/// slots were last brought to (unchanged by every read since), and whether
/// a read since found the parents changed.
#[derive(Default)]
struct Pending {
    still: Vec<bool>,
    reparented: bool,
}

#[derive(Clone, Copy)]
struct Slot {
    bits: u64,
    /// Whether the entity has `og.Transform` this frame.
    local_set: bool,
    /// Whether `world` holds its world matrix: kept from the last read
    /// while its transform, its ancestors' and the parents are unchanged.
    world_set: bool,
    /// Whether its transform differs from the last read's (or it is new).
    changed: bool,
}

#[derive(Clone, Copy)]
struct SlotData {
    /// Its position, quaternion and scale as last read, as the columns'
    /// bytes (12, 16 and 12).
    trs: [u8; 40],
    /// Its local matrix, made from `trs` when first composed after a
    /// change ([`SlotData::local`]): the raycast record reads every transform
    /// as a tick starts and composes nothing unless a system raycasts.
    local: Option<Mat4>,
    world: Mat4,
}

impl SlotData {
    /// The local matrix of `trs`, made once per change.
    fn local(&mut self) -> Mat4 {
        *self.local.get_or_insert_with(|| {
            let t = &self.trs;
            let x =
                |i: usize| f32::from_le_bytes([t[i * 4], t[i * 4 + 1], t[i * 4 + 2], t[i * 4 + 3]]);
            Mat4::from_scale_rotation_translation(
                Vec3::new(x(7), x(8), x(9)),
                Quat::from_xyzw(x(3), x(4), x(5), x(6)),
                Vec3::new(x(0), x(1), x(2)),
            )
        })
    }
}

const EMPTY_SLOT: Slot = Slot {
    bits: 0,
    local_set: false,
    world_set: false,
    changed: true,
};

const EMPTY_DATA: SlotData = SlotData {
    trs: [0; 40],
    local: None,
    world: Mat4::IDENTITY,
};

impl Hierarchy {
    pub(crate) fn of(world: &World) -> Hierarchy {
        let mut h = Hierarchy::default();
        h.refresh(world);
        h
    }

    /// Reads the world's transforms and parents again, keeping the
    /// allocations of the last frame and every world matrix that cannot
    /// have moved: an entity whose transform (bit for bit) and whose
    /// ancestors' transforms are as last read, under the same parents,
    /// keeps its world matrix, so a frame (or the raycast's tick record)
    /// where little moves composes only what moved. The ECS keeps no
    /// change ticks, so the columns are still read and compared.
    pub(crate) fn refresh(&mut self, world: &World) {
        self.capture(world);
        self.apply();
    }

    /// The first half of [`Hierarchy::refresh`]: copies the world's
    /// transform columns and parents, and leaves bringing the slots to
    /// them (a pass over every entity whose archetype changed) to the next
    /// question about an entity. The raycast record reads the world as
    /// every tick starts and most ticks ask it nothing (20,000 moving
    /// spheres: the per-entity pass was a tenth of the wasm work of a frame
    /// on the retired WebGL2 tier). Reads with no question between them add up: an archetype is
    /// still only when it was still at each of them.
    pub(crate) fn capture(&mut self, world: &World) {
        self.composed = 0;
        let mut pending = self.pending.take();
        let mut still = Vec::new();
        if let Ok(tid) = world.id_by_name("og.Transform") {
            let fields = ["position", "quaternion", "scale"];
            let refs: Vec<_> = fields
                .iter()
                .filter_map(|n| world.field_by_id(tid, n).ok())
                .collect();
            let q = QueryDesc {
                read: vec!["og.Transform".into()],
                ..QueryDesc::default()
            };
            if let (Ok(all), 3) = (world.query(&q), refs.len()) {
                for a in all {
                    let (Ok(_), Ok(_), Ok(_), Ok(pb), Ok(rb), Ok(sb)) = (
                        a.read::<Vec3>(refs[0]),
                        a.read::<Quat>(refs[1]),
                        a.read::<Vec3>(refs[2]),
                        a.read_bytes(refs[0]),
                        a.read_bytes(refs[1]),
                        a.read_bytes(refs[2]),
                    ) else {
                        continue;
                    };
                    let k = still.len();
                    if self.columns.len() <= k {
                        self.columns.push(Default::default());
                    }
                    let (ents, cols) = &mut self.columns[k];
                    let same = ents.as_slice() == a.entities()
                        && cols[0] == *pb
                        && cols[1] == *rb
                        && cols[2] == *sb;
                    if !same {
                        ents.clear();
                        ents.extend_from_slice(a.entities());
                        for (c, b) in cols.iter_mut().zip([&pb, &rb, &sb]) {
                            c.clear();
                            c.extend_from_slice(b);
                        }
                    }
                    let before = pending
                        .as_ref()
                        .is_none_or(|p| p.still.get(k) == Some(&true));
                    still.push(same && before);
                }
            }
        }
        self.columns.truncate(still.len());
        let reparented = self.read_parents(world);
        let reparented = reparented || pending.as_ref().is_some_and(|p| p.reparented);
        let p = pending.get_or_insert_with(Pending::default);
        p.still = still;
        p.reparented = reparented;
        self.pending = pending;
    }

    /// Whether a read of the world waits to be applied to the slots.
    #[cfg(test)]
    pub(crate) fn pending(&self) -> bool {
        self.pending.is_some()
    }

    /// The second half of [`Hierarchy::refresh`]: the slots brought to the
    /// last [`Hierarchy::capture`] (nothing when there is none pending).
    #[inline(never)]
    fn apply(&mut self) {
        let Some(Pending { still, reparented }) = self.pending.take() else {
            return;
        };
        self.loose.clear();
        self.walked = 0;
        let applied = std::mem::replace(&mut self.applied, still.len());
        if !reparented && still.len() == applied && still.iter().all(|&s| s) {
            // The same archetypes, entities, bytes and parents as the
            // slots were last brought to: every slot, its `local_set` and
            // its kept world matrix stand as they are (`changed` is only
            // read within this pass, which sets it for every slot first).
            // 20,000 still meshes walked 20,000 entities and every slot
            // three times here for nothing.
            return;
        }
        self.walked = self.slots.len();
        // `local_set` now says "had og.Transform at the last read".
        let mut was: Vec<bool> = std::mem::take(&mut self.was);
        was.clear();
        was.extend(self.slots.iter().map(|s| s.local_set));
        for s in &mut self.slots {
            s.local_set = false;
        }
        for ((ents, cols), &still) in self.columns.iter().zip(&still) {
            self.walked += ents.len();
            let (pb, rb, sb) = (&cols[0], &cols[1], &cols[2]);
            for (i, e) in ents.iter().enumerate() {
                let index = e.index() as usize;
                if still && was.get(index).copied().unwrap_or(false) {
                    // As last read, and the slot was this entity's then.
                    let slot = &mut self.slots[index];
                    slot.changed = false;
                    slot.local_set = true;
                    continue;
                }
                if index >= self.slots.len() {
                    self.slots.resize(index + 1, EMPTY_SLOT);
                    self.data.resize(index + 1, EMPTY_DATA);
                }
                let (slot, data) = (&mut self.slots[index], &mut self.data[index]);
                let bits = e.to_bits();
                let (pb, rb, sb) = (
                    &pb[i * 12..i * 12 + 12],
                    &rb[i * 16..i * 16 + 16],
                    &sb[i * 12..i * 12 + 12],
                );
                let same = was.get(index).copied().unwrap_or(false)
                    && slot.bits == bits
                    && data.trs[..12] == *pb
                    && data.trs[12..28] == *rb
                    && data.trs[28..] == *sb;
                slot.changed = !same;
                if !same {
                    slot.bits = bits;
                    data.trs[..12].copy_from_slice(pb);
                    data.trs[12..28].copy_from_slice(rb);
                    data.trs[28..].copy_from_slice(sb);
                    slot.world_set = false;
                    data.local = None;
                }
                slot.local_set = true;
            }
        }
        // A slot that lost its transform changed; one that never had one
        // did not.
        let mut moved = false;
        for (i, s) in self.slots.iter_mut().enumerate() {
            if !s.local_set {
                s.changed = was.get(i).copied().unwrap_or(false);
                s.world_set = false;
            }
            moved |= s.changed;
        }
        self.was = was;
        if reparented {
            // Every world matrix is composed again.
            for s in &mut self.slots {
                s.world_set = false;
            }
        } else if moved {
            // A kept child under an ancestor whose slot changed (moved, new,
            // gone) is composed again; an ancestor without og.Transform is
            // the identity, so the walk goes on through it.
            for &child in self.parent.keys() {
                let index = Entity::from_bits(child).index() as usize;
                if !self
                    .slots
                    .get(index)
                    .is_some_and(|s| s.world_set && s.bits == child)
                {
                    continue;
                }
                let mut cur = child;
                let mut depth = 0;
                let mut stale = false;
                while let Some(&p) = self.parent.get(&cur) {
                    depth += 1;
                    let changed = self
                        .slots
                        .get(Entity::from_bits(p).index() as usize)
                        .is_some_and(|s| s.changed);
                    if changed || depth > 256 || p == child {
                        stale = true;
                        break;
                    }
                    cur = p;
                }
                if stale {
                    self.slots[index].world_set = false;
                }
            }
        }
    }

    /// Reads og.Parent into `parent`; whether it differs from the last
    /// read. An archetype whose entities and column are as last read is
    /// not read again.
    fn read_parents(&mut self, world: &World) -> bool {
        let mut views = Vec::new();
        if let Ok(pid) = world.id_by_name("og.Parent")
            && let Ok(field) = world.field_by_id(pid, "entity")
        {
            let q = QueryDesc {
                read: vec!["og.Parent".into()],
                ..QueryDesc::default()
            };
            if let Ok(all) = world.query(&q) {
                for a in all {
                    if let Ok(b) = a.read_bytes(field) {
                        views.push((a.entities(), b));
                    }
                }
            }
        }
        let same = views.len() == self.parent_columns.len()
            && views
                .iter()
                .zip(&self.parent_columns)
                .all(|((e, b), (pe, pb))| *e == pe.as_slice() && **b == *pb.as_slice());
        if same {
            return false;
        }
        self.parent_columns.clear();
        let mut parent = IdMap::new();
        for (e, b) in &views {
            for (entity, p) in e.iter().zip(b.as_chunks::<8>().0) {
                let p = u64::from_le_bytes(*p);
                parent.insert(entity.to_bits(), p);
            }
            self.parent_columns.push((e.to_vec(), b.to_vec()));
        }
        let changed = parent != self.parent;
        self.parent = parent;
        changed
    }

    /// The slot of the entity `e` (bits), when it has `og.Transform`.
    fn slot(&mut self, e: u64) -> Option<(&mut Slot, &mut SlotData)> {
        let index = Entity::from_bits(e).index() as usize;
        let s = self.slots.get_mut(index)?;
        (s.local_set && s.bits == e).then(|| (s, &mut self.data[index]))
    }

    fn cached(&mut self, e: u64) -> Option<Mat4> {
        match self.slot(e) {
            Some((s, d)) => s.world_set.then_some(d.world),
            None => self.loose.get(&e).copied(),
        }
    }

    /// Whether `e` is `ancestor` or under it through og.Parent (bits).
    pub(crate) fn descends(&self, e: u64, ancestor: u64) -> bool {
        let mut cur = e;
        for _ in 0..=256 {
            if cur == ancestor {
                return true;
            }
            match self.parent.get(&cur) {
                Some(&p) => cur = p,
                None => return false,
            }
        }
        false
    }

    /// The world matrix of an entity's parent (the identity for a root).
    pub(crate) fn parent_world(&mut self, e: u64) -> Mat4 {
        match self.parent.get(&e).copied() {
            Some(p) => self.world_of(p),
            None => Mat4::IDENTITY,
        }
    }

    /// Blends every world matrix [`Hierarchy::world_of`] answers with
    /// `blend.prev`'s until [`Hierarchy::take_blend`].
    pub(crate) fn set_blend(&mut self, blend: Blend) {
        self.blend = Some(Box::new(blend));
    }

    /// Ends [`Hierarchy::set_blend`]: the previous tick's hierarchy back.
    pub(crate) fn take_blend(&mut self) -> Option<Hierarchy> {
        self.blend.take().map(|b| b.prev)
    }

    /// Whether `e` has og.Transform here.
    fn knows(&mut self, e: u64) -> bool {
        if self.pending.is_some() {
            self.apply();
        }
        self.slot(e).is_some()
    }

    /// The world matrix of an entity as drawn: its local transform after
    /// its parents', blended with the previous tick's on a frame between
    /// ticks ([`Hierarchy::set_blend`]).
    pub(crate) fn world_of(&mut self, e: u64) -> Mat4 {
        let m = self.tick_world_of(e);
        match self.blend.as_mut() {
            Some(b) => b.of(&self.parent, e, m),
            None => m,
        }
    }

    /// The world matrix of an entity at the tick: its local transform after its parents'.
    fn tick_world_of(&mut self, e: u64) -> Mat4 {
        // Checked in place: the raycast record asks once per target.
        if self.pending.is_some() {
            self.apply();
        }
        if let Some(m) = self.cached(e) {
            return m;
        }
        if !self.parent.contains_key(&e) {
            // A root: the chain below of one entity, without its vector.
            self.composed += 1;
            let mut m = Mat4::IDENTITY;
            match self.slot(e) {
                Some((s, d)) => {
                    m *= d.local();
                    d.world = m;
                    s.world_set = true;
                }
                None => {
                    m *= Mat4::IDENTITY;
                    self.loose.insert(e, m);
                }
            }
            return m;
        }
        let mut chain = vec![e];
        let mut cur = e;
        while let Some(&p) = self.parent.get(&cur) {
            if chain.contains(&p) || chain.len() > 256 {
                break;
            }
            chain.push(p);
            cur = p;
        }
        let mut m = Mat4::IDENTITY;
        for &id in chain.iter().rev() {
            if let Some(w) = self.cached(id) {
                m = w;
                continue;
            }
            self.composed += 1;
            match self.slot(id) {
                Some((s, d)) => {
                    m *= d.local();
                    d.world = m;
                    s.world_set = true;
                }
                None => {
                    m *= Mat4::IDENTITY;
                    self.loose.insert(id, m);
                }
            }
        }
        m
    }
}
