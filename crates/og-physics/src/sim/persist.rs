use super::*;

/// Whether a collider's shape is built from a registered asset (a mesh or a
/// height field), so a save carries its key instead of its geometry.
fn from_asset(c: &Collider) -> bool {
    matches!(c.shape, SHAPE_CONVEX | SHAPE_TRIMESH | SHAPE_HEIGHTFIELD)
        && c.mesh & crate::GENERATED_MESH == 0
}

fn asset_key(c: &Collider, scale: Vec3) -> ShapeKey {
    ShapeKey {
        shape: c.shape,
        mesh: c.mesh,
        scale: scale.abs().to_array().map(f32::to_bits),
    }
}

/// `sim`'s bytes without the geometry of its mesh, convex-hull and
/// height-field colliders: meshes are assets, so a save carries each such
/// shape's key and [`load`] rebuilds it from the asset. Everything else is
/// rapier's own serialisation, byte for byte.
pub(crate) fn save(sim: &Sim) -> (Vec<Hole>, Vec<u8>) {
    let fits = "a physics world always serializes into memory";
    let mut full = Vec::new();
    crate::encode(&mut full, sim).expect(fits);
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
    if keyed.is_empty() {
        return (Vec::new(), full);
    }
    // The colliders' bytes start after the world's fields before them
    // (bincode writes a struct's fields, and a tuple's, one after another):
    // measured by writing those fields again with the same encoder rather
    // than a size pass, which would compile their serialisation twice.
    let mut scratch = Vec::new();
    let before = (
        &w.gravity,
        &w.integration_parameters,
        &w.islands,
        &w.broad_phase,
        &w.narrow_phase,
        &w.bodies,
    );
    crate::encode(&mut scratch, &before).expect(fits);
    let start = scratch.len();
    scratch.clear();
    crate::encode(&mut scratch, &w.colliders).expect(fits);
    let end = start + scratch.len();
    let mut holes = Vec::new();
    let mut out = Vec::with_capacity(full.len());
    let mut at = start;
    out.extend_from_slice(&full[..start]);
    for (h, c) in w.colliders.iter() {
        let Some(key) = keyed.get(&h.into_raw_parts()) else {
            continue;
        };
        scratch.clear();
        crate::encode(&mut scratch, c.shared_shape()).expect(fits);
        let shape = &scratch;
        // The shape is the collider's second field: the first match past
        // the previous collider is it.
        let Some(found) = full[at..end]
            .windows(shape.len())
            .position(|bytes| bytes == shape.as_slice())
        else {
            continue;
        };
        out.extend_from_slice(&full[at..at + found]);
        holes.push(Hole {
            at: out.len() as u64,
            key: *key,
        });
        at += found + shape.len();
    }
    out.extend_from_slice(&full[at..]);
    (holes, out)
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
        let at = (hole.at as usize).clamp(from, bytes.len());
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
