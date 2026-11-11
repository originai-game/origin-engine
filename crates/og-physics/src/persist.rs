use super::*;

enum Saved {
    Empty,
    Pushes(Vec<Push>),
    #[cfg(feature = "physics3d")]
    D3(Box<dim3::Sim>, Vec<Push>, Option<Restored>),
    #[cfg(feature = "physics2d")]
    D2(Box<dim2::Sim>, Vec<Push>),
}

/// A restored 3D simulation's bytes, kept until the first step after the
/// restore: its asset shapes were built from the height fields registered
/// as it was decoded, which a world that ran on past an edit holds edited.
/// Until that step, a field registered with other heights (the terrain's
/// edit log replayed to the snapshot's) reloads the simulation from these
/// bytes over them ([`reload_restored`]) instead of rebuilding its
/// colliders, so the restored world steps as the snapshot's did.
#[cfg(feature = "physics3d")]
pub(crate) struct Restored {
    sim: Vec<u8>,
    holes: Vec<Hole>,
    sections: Vec<(u16, Vec<u8>)>,
}

pub(super) fn codec() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new().reject_trailing_bytes()
}

/// `value`'s bytes appended to `out` through [`codec`]. Everything a save
/// writes goes through this one encoder, so each type's serialisation is
/// compiled once: bincode's `serialize` and `serialized_size` would each
/// compile the whole rapier world's again (a size pass, a writer of
/// another type), tens of kilobytes of wasm (`the former test sample`).
pub(crate) fn encode<T: serde::Serialize + ?Sized>(
    out: &mut Vec<u8>,
    value: &T,
) -> bincode::Result<()> {
    use bincode::Options;
    codec().serialize_into(out, value)
}

/// A value [`encode`] wrote, alone in `bytes`.
#[cfg(feature = "physics3d")]
pub(crate) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    use bincode::Options;
    codec().deserialize(bytes)
}

/// What follows the profile tag in a saved `og.physics` block: the pushes
/// not yet applied, then the simulation's bytes without its asset shapes
/// and where they go (empty with tag 0, no simulation yet).
type Block = (Vec<Push>, Vec<Hole>, Vec<u8>);

/// After a restore and before the next step, height field `handle` was
/// registered with other heights: the restored simulation is decoded again
/// from its bytes over them (its shape caches for the field dropped), its
/// colliders, contacts and handles as the snapshot had them. `false` when
/// no restored simulation built a shape from the field: the caller marks
/// it stale as usual.
#[cfg(feature = "physics3d")]
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
    shared.sim3 = Some(Box::new(sim));
    shared.restored = Some(source);
    Ok(true)
}

impl StateBlock for PhysicsState {
    fn save(&self, out: &mut Vec<u8>) {
        let shared = lock(&self.0);
        let pushes = shared.pushes.clone();
        let mut sections = Vec::new();
        let block: Option<(u8, Block)> = match () {
            #[cfg(feature = "physics3d")]
            _ if shared.sim3.is_some() => {
                let sim = shared.sim3.as_deref().unwrap();
                sections = sim.sections();
                let (holes, bytes) = dim3::save(sim);
                Some((PROFILE_3D as u8, (pushes, holes, bytes)))
            }
            #[cfg(feature = "physics2d")]
            _ if shared.sim2.is_some() => {
                let sim = shared.sim2.as_deref().unwrap();
                sections = sim.sections();
                let (holes, bytes) = dim2::save(sim);
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
            #[cfg(feature = "physics3d")]
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
            #[cfg(feature = "physics2d")]
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

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut shared = lock(&self.0);
        // The restored colliders are as saved, whatever was registered before.
        #[cfg(feature = "physics3d")]
        {
            shared.restored = None;
        }
        shared.stale_meshes.clear();
        shared.field_edits.clear();
        shared.pushes.clear();
        #[cfg(feature = "physics3d")]
        {
            shared.sim3 = None;
        }
        #[cfg(feature = "physics2d")]
        {
            shared.sim2 = None;
        }
        let saved = decoded.map(|d| {
            *d.downcast::<Saved>()
                .expect("decoded by PhysicsState::decode")
        });
        match saved {
            None | Some(Saved::Empty) => {}
            Some(Saved::Pushes(p)) => shared.pushes = p,
            #[cfg(feature = "physics3d")]
            Some(Saved::D3(sim, p, source)) => {
                shared.sim3 = Some(sim);
                shared.pushes = p;
                shared.restored = source;
            }
            #[cfg(feature = "physics2d")]
            Some(Saved::D2(sim, p)) => {
                shared.sim2 = Some(sim);
                shared.pushes = p;
            }
        }
    }

    /// The simulation in place ([`dim3::Sim::shift`]) and the points of the
    /// pushes not yet applied. The query view recorded for this tick was
    /// read before the shift: dropped, so the next query records again.
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut shared = lock(&self.0);
        #[cfg(feature = "physics3d")]
        {
            // Moved in place: its bytes are no longer the simulation's.
            shared.restored = None;
        }
        #[cfg(feature = "physics3d")]
        if let Some(sim) = shared.sim3.as_deref_mut() {
            sim.shift(s);
        }
        #[cfg(feature = "physics2d")]
        if let Some(sim) = shared.sim2.as_deref_mut() {
            sim.shift(s);
        }
        for p in &mut shared.pushes {
            p.point = p.point.map(|q| s.moved(q));
        }
        shared.recorded = None;
        shared.recorded_query = None;
    }
}
