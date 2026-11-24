use super::*;

enum Saved {
    Empty,
    Pushes(Vec<Push>),
    D3(Box<dim3::Sim>, Vec<Push>, Option<Restored>),
    D2(Box<dim2::Sim>, Vec<Push>),
}

/// A restored 3D simulation's bytes, kept until the first step after the
/// restore: its asset shapes were built from the height fields registered
/// as it was decoded, which a world that ran on past an edit holds edited.
/// Until that step, a field registered with other heights (the terrain's
/// edit log replayed to the snapshot's) reloads the simulation from these
/// bytes over them ([`reload_restored`]) instead of rebuilding its
/// colliders, so the restored world steps as the snapshot's did.
pub(crate) struct Restored {
    sim: Vec<u8>,
    holes: Vec<Hole>,
    sections: Vec<(u16, Vec<u8>)>,
}

pub(super) use og_physics_data::codec;
pub(crate) use og_physics_data::encode;

/// What follows the profile tag in a saved `og.physics` block: the pushes
/// not yet applied, then the simulation's bytes without its asset shapes
/// and where they go (empty with tag 0, no simulation yet).
type Block = (Vec<Push>, Vec<Hole>, Vec<u8>);

const FORMAT: u8 = 2;

/// After a restore and before the next step, height field `handle` was
/// registered with other heights: the restored simulation is decoded again
/// from its bytes over them (its shape caches for the field dropped), its
/// colliders, contacts and handles as the snapshot had them. `false` when
/// no restored simulation built a shape from the field: the caller marks
/// it stale as usual.
pub(super) fn reload_restored(shared: &mut Shared, handle: u64) -> Result<bool> {
    let Some(source) = shared.restored.take() else {
        return Ok(false);
    };
    if !source.holes.iter().any(|h| h.key.mesh == handle) {
        shared.restored = Some(source);
        return Ok(false);
    }
    shared.shapes3.retain(|k, _| k.mesh != handle);
    shared.chunks3.retain(|k, _| k.mesh != handle);
    let sections: Vec<(u16, &[u8])> = source
        .sections
        .iter()
        .map(|(k, b)| (*k, b.as_slice()))
        .collect();
    let mut sim = outside(|| dim3::load(&source.sim, &source.holes, shared))?;
    sim.restore_sections(&sections)?;
    shared.sim3 = Some(Arc::new(Mutex::new(Box::new(sim))));
    shared.restored = Some(source);
    Ok(true)
}

impl StateBlock for PhysicsState {
    fn save(&self, out: &mut Vec<u8>) {
        out.push(FORMAT);
        let shared = lock(&self.0);
        let pushes = shared.pushes.clone();
        let mut sections = Vec::new();
        let block: Option<(u8, Block)> = match () {
            _ if shared.sim3.is_some() => {
                let sim = lock(shared.sim3.as_deref().unwrap());
                sections = sim.sections();
                let (holes, bytes) = dim3::save(&sim);
                Some((PROFILE_3D as u8, (pushes, holes, bytes)))
            }
            _ if shared.sim2.is_some() => {
                let sim = lock(shared.sim2.as_deref().unwrap());
                sections = sim.sections();
                let (holes, bytes) = dim2::save(&sim);
                Some((PROFILE_2D as u8, (pushes, holes, bytes)))
            }
            _ if !pushes.is_empty() => Some((0, (pushes, Vec::new(), Vec::new()))),
            _ => None,
        };
        if let Some((tag, block)) = block {
            out.push(tag);
            encode(out, &block).expect("a physics world always serializes into memory");
            og_ecs::trailer::write(out, &sections);
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        use bincode::Options;
        let parse = |e: bincode::Error| {
            Error::new(
                ErrorKind::Parse,
                format!("the rapier world does not decode: {e}"),
            )
        };
        let Some((&FORMAT, bytes)) = bytes.split_first() else {
            return Err(Error::new(
                ErrorKind::Parse,
                "incompatible og.physics state format; old saves are not supported",
            ));
        };
        let Some((&tag, rest)) = bytes.split_first() else {
            return Ok(Box::new(Saved::Empty));
        };
        let mut reader = rest;
        let (pushes, holes, sim): Block = codec()
            .allow_trailing_bytes()
            .deserialize_from(&mut reader)
            .map_err(parse)?;
        let sections = og_ecs::trailer::read("og.physics", reader)?;
        // The asset shapes are rebuilt from the registered meshes, as a
        // step builds them (outside the step, like a query view).
        let saved = match tag as u32 {
            0 => Saved::Pushes(pushes),
            PROFILE_3D => {
                let mut built = outside(|| dim3::load(&sim, &holes, &mut lock(&self.0)))?;
                built.restore_sections(&sections)?;
                let fields = holes
                    .iter()
                    .any(|h| h.key.shape & 0xff == SHAPE_HEIGHTFIELD);
                let source = fields.then(|| Restored {
                    sim,
                    holes,
                    sections: sections.iter().map(|&(k, b)| (k, b.to_vec())).collect(),
                });
                Saved::D3(Box::new(built), pushes, source)
            }
            PROFILE_2D => {
                let mut sim = outside(|| dim2::load(&sim, &holes, &mut lock(&self.0)))?;
                sim.restore_sections(&sections)?;
                Saved::D2(Box::new(sim), pushes)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("a {tag}D physics world; this engine lacks that profile"),
                ));
            }
        };
        Ok(Box::new(saved))
    }

    /// The saved simulation by entity, decoded as [`Self::decode`] decodes
    /// it (applied to nothing): each entity's bodies, colliders, joints,
    /// vehicles, ragdolls, ropes, cloths, fluids, soft volumes, garments,
    /// water and contacts ([`dim3::Sim::form`]), and in `global` the
    /// profile, gravity, integration parameters and the pushes not yet
    /// applied (each push's fields as `push {i}.…`, its body an entity).
    fn form(&self, bytes: &[u8]) -> Result<og_core::block_form::BlockForm> {
        use og_core::block_form::{BlockForm, Field, Records};
        let decoded = self.decode(bytes)?;
        let saved = *decoded
            .downcast::<Saved>()
            .expect("decoded by PhysicsState::decode");
        let mut out = Records::default();
        let profile = bytes.get(1).map_or(0, |&t| u32::from(t));
        out.global.push(("profile".to_owned(), Field::of(&profile)));
        let pushes = match saved {
            Saved::Empty => Vec::new(),
            Saved::Pushes(p) => p,
            Saved::D3(sim, p, _) => {
                sim.form(&mut out);
                p
            }
            Saved::D2(sim, p) => {
                sim.form(&mut out);
                p
            }
        };
        out.global
            .push(("pushes".to_owned(), Field::of(&pushes.len())));
        for (i, p) in pushes.iter().enumerate() {
            let name = |f: &str| format!("push {i}.{f}");
            out.global.extend([
                (name("entity"), Field::Entity(p.entity)),
                (name("kind"), Field::of(&p.kind)),
                (name("vector"), Field::of(&p.vector)),
                (name("point"), Field::of(&p.point)),
            ]);
        }
        Ok(BlockForm::Entities(out))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut shared = lock(&self.0);
        // The restored colliders are as saved, whatever was registered before.
        shared.restored = None;
        shared.stale_meshes.clear();
        shared.field_edits.clear();
        shared.pushes.clear();
        shared.ragdoll_targets.clear();
        shared.garment_poses.clear();
        shared.sim3 = None;
        shared.sim2 = None;
        let saved = decoded.map(|d| {
            *d.downcast::<Saved>()
                .expect("decoded by PhysicsState::decode")
        });
        match saved {
            None | Some(Saved::Empty) => {}
            Some(Saved::Pushes(p)) => shared.pushes = p,
            Some(Saved::D3(sim, p, source)) => {
                shared.sim3 = Some(Arc::new(Mutex::new(sim)));
                shared.pushes = p;
                shared.restored = source;
            }
            Some(Saved::D2(sim, p)) => {
                shared.sim2 = Some(Arc::new(Mutex::new(sim)));
                shared.pushes = p;
            }
        }
    }

    /// The simulation in place ([`dim3::Sim::shift`]) and the points of the
    /// pushes not yet applied. Queries immediately read the shifted world.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut shared = lock(&self.0);
        // Moved in place: its bytes are no longer the simulation's.
        shared.restored = None;
        if let Some(sim) = shared.sim3.as_deref() {
            lock(sim).shift(s);
        }
        if let Some(sim) = shared.sim2.as_deref() {
            lock(sim).shift(s);
        }
        for p in &mut shared.pushes {
            p.point = p.point.map(|q| s.moved(q));
        }
        for poses in shared.ragdoll_targets.values_mut() {
            for (p, _) in poses {
                *p = s.moved(*p);
            }
        }
        for pose in shared.garment_poses.values_mut() {
            for p in &mut pose.targets {
                *p = s.moved(*p);
            }
        }
    }
}

/// Single-pass bincode serializer that strips the `shape` field of asset-backed
/// `rapier::Collider` entries inside `ColliderSet::colliders` and records each
/// cut point as a [`Hole`]. Never mutates the simulation and never encodes
/// multi-MB asset meshes/heightfields only to slice them out afterward.
pub(crate) fn encode_stripped<T: Serialize>(
    value: &T,
    slot_keys: &[Option<ShapeKey>],
) -> (Vec<Hole>, Vec<u8>) {
    let mut ser = Stripper {
        out: Vec::new(),
        holes: Vec::new(),
        slot_keys,
        slot_cursor: 0,
        in_colliders: false,
    };
    value
        .serialize(&mut ser)
        .expect("a physics world always serializes into memory");
    debug_assert!(
        slot_keys.is_empty() || ser.slot_cursor == slot_keys.len(),
        "ColliderSet slot count mismatch: visited {}, expected {}",
        ser.slot_cursor,
        slot_keys.len()
    );
    (ser.holes, ser.out)
}

struct Stripper<'a> {
    out: Vec<u8>,
    holes: Vec<Hole>,
    slot_keys: &'a [Option<ShapeKey>],
    slot_cursor: usize,
    in_colliders: bool,
}

impl Stripper<'_> {
    fn inner(&mut self) -> bincode::Serializer<&mut Vec<u8>, impl bincode::Options> {
        bincode::Serializer::new(&mut self.out, codec())
    }
}

enum StripStruct<'a, 'b> {
    Plain(&'b mut Stripper<'a>),
    ColliderSet(&'b mut Stripper<'a>),
    Collider {
        ser: &'b mut Stripper<'a>,
        key: Option<ShapeKey>,
    },
}

impl<'a, 'b> serde::Serializer for &'b mut Stripper<'a> {
    type Ok = ();
    type Error = bincode::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = StripStruct<'a, 'b>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> bincode::Result<()> {
        serde::Serializer::serialize_bool(&mut self.inner(), v)
    }
    fn serialize_i8(self, v: i8) -> bincode::Result<()> {
        serde::Serializer::serialize_i8(&mut self.inner(), v)
    }
    fn serialize_i16(self, v: i16) -> bincode::Result<()> {
        serde::Serializer::serialize_i16(&mut self.inner(), v)
    }
    fn serialize_i32(self, v: i32) -> bincode::Result<()> {
        serde::Serializer::serialize_i32(&mut self.inner(), v)
    }
    fn serialize_i64(self, v: i64) -> bincode::Result<()> {
        serde::Serializer::serialize_i64(&mut self.inner(), v)
    }
    fn serialize_i128(self, v: i128) -> bincode::Result<()> {
        serde::Serializer::serialize_i128(&mut self.inner(), v)
    }
    fn serialize_u8(self, v: u8) -> bincode::Result<()> {
        serde::Serializer::serialize_u8(&mut self.inner(), v)
    }
    fn serialize_u16(self, v: u16) -> bincode::Result<()> {
        serde::Serializer::serialize_u16(&mut self.inner(), v)
    }
    fn serialize_u32(self, v: u32) -> bincode::Result<()> {
        serde::Serializer::serialize_u32(&mut self.inner(), v)
    }
    fn serialize_u64(self, v: u64) -> bincode::Result<()> {
        serde::Serializer::serialize_u64(&mut self.inner(), v)
    }
    fn serialize_u128(self, v: u128) -> bincode::Result<()> {
        serde::Serializer::serialize_u128(&mut self.inner(), v)
    }
    fn serialize_f32(self, v: f32) -> bincode::Result<()> {
        serde::Serializer::serialize_f32(&mut self.inner(), v)
    }
    fn serialize_f64(self, v: f64) -> bincode::Result<()> {
        serde::Serializer::serialize_f64(&mut self.inner(), v)
    }
    fn serialize_char(self, v: char) -> bincode::Result<()> {
        serde::Serializer::serialize_char(&mut self.inner(), v)
    }
    fn serialize_str(self, v: &str) -> bincode::Result<()> {
        serde::Serializer::serialize_str(&mut self.inner(), v)
    }
    fn serialize_bytes(self, v: &[u8]) -> bincode::Result<()> {
        serde::Serializer::serialize_bytes(&mut self.inner(), v)
    }
    fn serialize_none(self) -> bincode::Result<()> {
        serde::Serializer::serialize_none(&mut self.inner())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> bincode::Result<()> {
        self.out.push(1);
        v.serialize(self)
    }
    fn serialize_unit(self) -> bincode::Result<()> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> bincode::Result<()> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> bincode::Result<()> {
        serde::Serializer::serialize_unit_variant(&mut self.inner(), name, variant_index, variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> bincode::Result<()> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> bincode::Result<()> {
        serde::Serializer::serialize_unit_variant(&mut self.inner(), name, variant_index, variant)?;
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> bincode::Result<Self::SerializeSeq> {
        let _ = serde::Serializer::serialize_seq(&mut self.inner(), len)?;
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> bincode::Result<Self::SerializeTuple> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> bincode::Result<Self::SerializeTupleStruct> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> bincode::Result<Self::SerializeTupleVariant> {
        serde::Serializer::serialize_unit_variant(&mut self.inner(), name, variant_index, variant)?;
        Ok(self)
    }
    fn serialize_map(self, len: Option<usize>) -> bincode::Result<Self::SerializeMap> {
        let _ = serde::Serializer::serialize_map(&mut self.inner(), len)?;
        Ok(self)
    }
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> bincode::Result<Self::SerializeStruct> {
        if !self.slot_keys.is_empty() {
            if name == "ColliderSet" {
                return Ok(StripStruct::ColliderSet(self));
            }
            if self.in_colliders && name == "Collider" {
                let key = self.slot_keys[self.slot_cursor];
                self.slot_cursor += 1;
                return Ok(StripStruct::Collider { ser: self, key });
            }
        }
        Ok(StripStruct::Plain(self))
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> bincode::Result<Self::SerializeStructVariant> {
        serde::Serializer::serialize_unit_variant(&mut self.inner(), name, variant_index, variant)?;
        Ok(self)
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl serde::ser::SerializeSeq for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleStruct for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeTupleVariant for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeMap for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> bincode::Result<()> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeStructVariant for &mut Stripper<'_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> bincode::Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}

impl serde::ser::SerializeStruct for StripStruct<'_, '_> {
    type Ok = ();
    type Error = bincode::Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> bincode::Result<()> {
        match self {
            StripStruct::Plain(ser) => value.serialize(&mut **ser),
            StripStruct::ColliderSet(ser) => {
                let was = ser.in_colliders;
                if key == "colliders" {
                    ser.in_colliders = true;
                }
                let res = value.serialize(&mut **ser);
                ser.in_colliders = was;
                res
            }
            StripStruct::Collider { ser, key: shape } => {
                if key == "shape"
                    && let Some(k) = *shape
                {
                    ser.holes.push(Hole {
                        at: ser.out.len() as u64,
                        key: k,
                    });
                    return Ok(());
                }
                value.serialize(&mut **ser)
            }
        }
    }
    fn end(self) -> bincode::Result<()> {
        Ok(())
    }
}
