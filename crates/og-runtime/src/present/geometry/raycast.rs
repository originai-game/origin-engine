//! og.render.raycast: a BVH per geometry over the meshes drawn as the tick started.

use super::*;

/// The BVH of a mesh, built now when it is missing or older than the mesh.
pub(super) fn bvh_of(
    g: &mut GeoState,
    scene: &og_render::RenderScene,
    mesh: MeshHandle,
) -> Option<Rc<Bvh>> {
    let rev = scene.mesh_revision(mesh.index());
    if let Some((r, b)) = g.bvh.get(&mesh.index())
        && *r == rev
    {
        return Some(b.clone());
    }
    let b = Rc::new(Bvh::build(scene.mesh(mesh)?));
    g.bvh.insert(mesh.index(), (rev, b.clone()));
    Some(b)
}

/// A loaded model's primitive takes the BVH its cook built (over the mesh
/// the scene now holds at `mesh`), so no raycast builds one; a primitive
/// cooked with `raycast false` keeps the lazy build.
/// Drops what was cached for a mesh (its raycast BVH, its bounds): a
/// released uncooked model's meshes (og.load.release).
pub(crate) fn forget_mesh(rs: &mut RenderState, mesh: MeshHandle) {
    rs.geo.bvh.remove(&mesh.index());
    rs.geo.bounds.remove(&mesh.index());
}

pub(crate) fn adopt_cooked_bvh(
    rs: &mut RenderState,
    mesh: MeshHandle,
    prim: &og_bundle::runtime::CookedPrimitive,
) -> Result<(), String> {
    let RenderState { geo: g, scene, .. } = rs;
    let Some(data) = scene.mesh(mesh) else {
        return Ok(());
    };
    let Some(bvh) = prim.bvh(data) else {
        return Ok(());
    };
    let bvh = bvh?;
    g.bvh.insert(
        mesh.index(),
        (scene.mesh_revision(mesh.index()), Rc::new(bvh)),
    );
    Ok(())
}

/// Every og.Mesh and og.Model part as drawn from the world now, with `h`
/// read again (its slots brought to the read when the record is first
/// composed, [`Hierarchy::capture`]): the og.Mesh rows in query order (`kept`'s while their
/// columns are unchanged), and the og.Model parts in entity order into
/// `out` (cleared).
pub(super) fn drawn(
    world: &World,
    rs: &RenderState,
    h: &mut Hierarchy,
    kept: &mut Meshes,
    out: &mut Vec<Drawn>,
) -> Rc<Vec<Drawn>> {
    use crate::host::codec::{Rec, Slot};
    out.clear();
    h.capture(world);
    let meshes = match &rs.draw.mesh {
        Some(layout) if kept.still(world, &layout.schema, rs.geometries.len()) => kept.list.clone(),
        Some(layout) => {
            let mut list = Vec::new();
            for r in crate::present::extract::mesh_rows(world, layout) {
                let Some(&mesh) = rs.geometries.get(r.geometry) else {
                    continue;
                };
                list.push(Drawn {
                    entity: r.entity,
                    geometry: r.geometry,
                    mesh,
                    part: None,
                    visible: r.visible,
                    layers: r.layers,
                });
            }
            kept.list = Rc::new(list);
            kept.list.clone()
        }
        None => Rc::new(Vec::new()),
    };
    if let Some(layout) = &rs.draw.model {
        // Read in place, as extraction reads them (decoding each row to
        // JSON was half of this read).
        let (asset, visible, layers) = (
            Slot::of(layout, "asset"),
            Slot::of(layout, "visible"),
            Slot::of(layout, "layers"),
        );
        let rows = crate::present::extract::column_rows(world, layout);
        let mut rows: Vec<_> = rows.iter().collect();
        og_core::sort::sort_by_key(&mut rows, |(e, _)| og_ecs::Entity::from_bits(*e).index());
        // Each part's geometry index through a map (its first position, as
        // a search gave): a search per part made this read, every tick,
        // parts × geometries (Daguanyuan, native: a 8.7 ms tick, 1.8 ms after).
        let mut geometry_of = std::collections::HashMap::with_capacity(rs.geometries.len());
        for (i, g) in rs.geometries.iter().enumerate() {
            geometry_of.entry(*g).or_insert(i);
        }
        for (e, v) in &rows {
            let Some(parts) = rs.models.get(&(v.num(&asset).unwrap_or(0.0) as u64)) else {
                continue;
            };
            let visible = v.flag(&visible).unwrap_or(true);
            let layers = v.uint(&layers).map_or(1, |x| x as u32);
            for &(mesh, _, local) in parts.iter() {
                out.push(Drawn {
                    entity: *e,
                    geometry: geometry_of.get(&mesh).copied().unwrap_or(usize::MAX),
                    mesh,
                    part: Some(local),
                    visible,
                    layers,
                });
            }
        }
    }
    meshes
}

/// The targets of `drawn`, their world matrices from `h`.
pub(super) fn compose(h: &mut Hierarchy, drawn: &[Drawn]) -> Vec<Target> {
    drawn
        .iter()
        .map(|d| {
            let m = h.world_of(d.entity);
            Target {
                entity: d.entity,
                geometry: d.geometry,
                mesh: d.mesh,
                transform: d.part.map_or(m, |local| m * local),
                visible: d.visible,
                layers: d.layers,
            }
        })
        .collect()
}

/// The probe's tick start: the drawn meshes as the tick starts, for a
/// raycast from inside a system. Their world matrices wait for the first
/// such raycast ([`Record`]).
pub(crate) fn before_tick(eng: &Engine, world: &mut World) {
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let tick = world.time().tick;
    let mut models = match rs.geo.record.take() {
        Some((_, Record::Drawn(_, models))) => models,
        _ => Vec::new(),
    };
    let mut h = std::mem::take(&mut rs.geo.hierarchy);
    let mut kept = std::mem::take(&mut rs.geo.meshes);
    let meshes = drawn(world, &rs, &mut h, &mut kept, &mut models);
    rs.geo.hierarchy = h;
    rs.geo.meshes = kept;
    rs.geo.record = Some((tick, Record::Drawn(meshes, models)));
}

pub fn og_render_raycast(
    world: u32,
    ray: String,
    desc: Option<String>,
) -> Result<Vec<crate::host::wire::GeometryHit>, String> {
    (|| {
        let eng = engine(world)?;
        let r: wire::Ray = wire::arg(Some(&ray), "ray")?;
        let d: wire::RaycastGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let (origin, dir) = (r.origin, r.direction);
        if !origin.is_finite() || !dir.is_finite() || dir.length_squared() < 1e-20 {
            return Err(
                "the ray's origin and direction must be finite and its direction not zero".into(),
            );
        }
        let dir = dir.normalize();
        let max = d.max_distance;
        let max = if max > 0.0 && max.is_finite() {
            max
        } else {
            f32::INFINITY
        };
        let (first, invisible) = (d.first, d.invisible);
        // Three.js layers: a mesh is tested when its mask shares a bit
        // (`crate::present::views`).
        let layers = d.layers;
        let list: Rc<Vec<Target>> = if running(eng.id).is_some() {
            let mut rs = eng.render.borrow_mut();
            let GeoState {
                record, hierarchy, ..
            } = &mut rs.geo;
            match record {
                Some((_, r)) => r.targets(hierarchy),
                None => Rc::new(Vec::new()),
            }
        } else {
            let mut rs = eng.render.borrow_mut();
            let mut h = std::mem::take(&mut rs.geo.now);
            let mut models = Vec::new();
            let meshes = drawn(
                eng.world()?,
                &rs,
                &mut h,
                &mut Meshes::default(),
                &mut models,
            );
            let mut targets = compose(&mut h, &meshes);
            targets.extend(compose(&mut h, &models));
            rs.geo.now = h;
            Rc::new(targets)
        };
        let mut rs = eng.render.borrow_mut();
        let RenderState { geo: g, scene, .. } = &mut *rs;
        let mut hits = Vec::new();
        let mut reach = max;
        for t in list
            .iter()
            .filter(|t| (t.visible || invisible) && t.layers & layers != 0)
        {
            let det = t.transform.determinant();
            if !det.is_finite() || det.abs() < 1e-12 {
                continue;
            }
            let Some(bvh) = bvh_of(g, scene, t.mesh) else {
                continue;
            };
            let inv = t.transform.inverse();
            // Not normalised: t along it is the world distance.
            let lo = inv.transform_point3(origin);
            let ld = inv.transform_vector3(dir);
            let normal_m = inv.transpose();
            for hit in bvh.raycast(lo, ld, reach, first) {
                let n = normal_m.transform_vector3(hit.normal).normalize_or_zero();
                let p = origin + dir * hit.t;
                hits.push((
                    hit.t,
                    wire::GeometryHit {
                        entity: t.entity,
                        geometry: if t.geometry == usize::MAX {
                            0
                        } else {
                            assets::handle(t.geometry)
                        },
                        distance: hit.t,
                        point: p,
                        normal: n,
                        uv: Vec2::from_array(hit.uv),
                        face_index: hit.face,
                    },
                ));
                if first {
                    reach = reach.min(hit.t);
                }
            }
        }
        og_core::sort::sort_by(&mut hits, |a, b| a.0.total_cmp(&b.0));
        if first {
            hits.truncate(1);
        }
        Ok(hits.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
    })()
}
