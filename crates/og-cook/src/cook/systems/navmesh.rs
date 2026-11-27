//! Checked TS navigation declarations bake through the shared native kernels.
//! Geometry is captured from imported models and scene snapshots; planning and
//! execution never reopen author files. Terrain consumers share its surface.
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::cook::{formats::geometry, source_content::Graph, source_schemas::world::bake as input};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::ogt::{Block, Document};
use std::collections::BTreeSet;

pub(crate) struct Prepared {
    input: input::NavmeshInput,
    origin: crate::cook::generator::source::Diagnostic,
    triangles: Vec<[Vec3; 3]>,
    notes: Vec<String>,
    dependencies: BTreeSet<String>,
    hash: u64,
}

impl Prepared {
    pub(crate) fn new(
        input: input::NavmeshInput,
        origin: crate::cook::generator::source::Diagnostic,
    ) -> Self {
        Self {
            input,
            origin,
            triangles: Vec::new(),
            notes: Vec::new(),
            dependencies: BTreeSet::new(),
            hash: 0,
        }
    }

    fn error(&self, message: String) -> CookError {
        CookError(
            crate::cook::generator::source::Diagnostic {
                message,
                ..self.origin.clone()
            }
            .to_string(),
        )
    }
}

/// Capture geometry only after generated scenes and all model imports exist.
/// The scene domain still owns its text input until its separate cutover.
pub(crate) fn bind(sources: &mut [Source], graph: &Graph) -> Result<(), CookError> {
    let pending: Vec<_> = sources
        .iter_mut()
        .enumerate()
        .filter_map(|(index, source)| source.navmesh.take().map(|p| (index, p)))
        .collect();
    for (index, mut prepared) in pending {
        let source = &sources[index];
        (|| {
            let input = &prepared.input;
            let mut dependencies = BTreeSet::new();
            let mut triangles = Vec::new();
            let mut notes = Vec::new();
            let models = |id: &str, _: Option<&str>| {
                let source = sources
                    .iter()
                    .find(|s| s.rel == id)
                    .ok_or_else(|| format!("model '{id}' has not been imported"))?;
                let gltf = source
                    .gltf
                    .as_ref()
                    .ok_or_else(|| format!("model '{id}' has no checked mesh"))?;
                super::model::operand_geometry(gltf, None)
            };
            if let Some(scene) = &input.scene {
                let asset = &graph.outputs[&scene.id].asset;
                dependencies.insert(asset.clone());
                let scene = sources
                    .iter()
                    .find(|s| &s.rel == asset)
                    .ok_or_else(|| format!("scene '{asset}' has not been captured"))?;
                if let Some(error) = &scene.error {
                    return Err(error.clone());
                }
                let doc = Document::parse(&String::from_utf8_lossy(&scene.bytes))
                    .map_err(|e| format!("{asset}: {e}"))?;
                scene_triangles(
                    &doc.body,
                    Mat4::IDENTITY,
                    &models,
                    &mut dependencies,
                    &mut triangles,
                    &mut notes,
                )?;
            }
            for b in &input.boxes {
                let m = Mat4::from_rotation_translation(
                    rotation(b.quaternion)?,
                    vector(b.center, Vec3::ZERO),
                );
                triangles.extend(
                    og_ai::nav::bake::box_triangles(Vec3::ZERO, vector(b.size, Vec3::ONE) * 0.5)
                        .into_iter()
                        .map(|t| t.map(|v| m.transform_point3(v))),
                );
            }
            for model in &input.models {
                let asset = &graph.outputs[&model.model.id].asset;
                dependencies.insert(asset.clone());
                let placement = geometry::placement(model.transform.as_ref())?;
                model_triangles(asset, placement.matrix().as_mat4(), &models, &mut triangles)?;
            }
            let mut hash = og_core::hash::StableHasher::new();
            for value in triangles.iter().flatten().flat_map(|v| v.to_array()) {
                hash.write_u32(value.to_bits());
            }
            for terrain in &input.terrains {
                dependencies.insert(terrain.terrain.id.clone());
                hash.write_u64(super::terrain::get(source, &terrain.terrain.id)?.hash);
            }
            prepared.triangles = triangles;
            prepared.notes = notes;
            prepared.dependencies = dependencies;
            prepared.hash = hash.finish();
            Ok(())
        })()
        .map_err(|e| prepared.error(e))?;
        sources[index].navmesh = Some(prepared);
    }
    Ok(())
}

fn vector(value: Option<[f32; 3]>, default: Vec3) -> Vec3 {
    value.map_or(default, Vec3::from_array)
}

fn rotation(value: Option<[f32; 4]>) -> Result<Quat, String> {
    let q = value.map_or(Quat::IDENTITY, Quat::from_array).as_dquat();
    if q.length_squared() < 1e-12 {
        return Err("quaternion is not a rotation (zero length)".into());
    }
    Ok(q.normalize().as_quat())
}

fn model_triangles(
    id: &str,
    matrix: Mat4,
    models: &geometry::Models<'_>,
    out: &mut Vec<[Vec3; 3]>,
) -> Result<(), String> {
    let mesh = models(id, None)?;
    out.extend(
        mesh.indices.as_chunks::<3>().0.iter().map(|t| {
            t.map(|i| matrix.transform_point3(Vec3::from_array(mesh.positions[i as usize])))
        }),
    );
    Ok(())
}

// Only the scene reader uses internal blocks. Navmesh has no text frontend.
fn string_of(b: &Block, key: &str) -> Option<String> {
    b.get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn numbers<const N: usize>(b: Option<&Block>, key: &str) -> Option<[f32; N]> {
    let e = b?.get(key)?;
    let values: Vec<_> = e
        .values
        .iter()
        .map(|v| v.as_number().map(|n| n.as_f32()))
        .collect::<Option<_>>()?;
    values.try_into().ok()
}

fn scene_triangles(
    b: &Block,
    parent: Mat4,
    models: &geometry::Models<'_>,
    dependencies: &mut BTreeSet<String>,
    out: &mut Vec<[Vec3; 3]>,
    notes: &mut Vec<String>,
) -> Result<(), String> {
    for e in b.get_all("entity") {
        let Some(block) = &e.block else { continue };
        let t = block.get("Transform").and_then(|t| t.block.as_ref());
        let m = parent
            * Mat4::from_scale_rotation_translation(
                vector(numbers(t, "scale"), Vec3::ONE),
                rotation(numbers(t, "quaternion"))?,
                vector(numbers(t, "position"), Vec3::ZERO),
            );
        let dynamic = block
            .get("RigidBody")
            .and_then(|r| r.block.as_ref())
            .is_some_and(|r| numbers::<1>(Some(r), "kind").unwrap_or([0.0])[0] == 0.0);
        if let Some(model) = string_of(block, "model")
            && !dynamic
        {
            dependencies.insert(model.clone());
            model_triangles(&model, m, models, out)?;
        }
        if let Some(c) = block.get("Collider").and_then(|c| c.block.as_ref())
            && !dynamic
        {
            let scalar = |key, default| numbers::<1>(Some(c), key).map_or(default, |v| v[0]);
            let half = match scalar("shape", 0.0) as u32 {
                0 => vector(numbers(Some(c), "size"), Vec3::ONE) * 0.5,
                1 => Vec3::splat(scalar("radius", 0.5)),
                shape @ (2 | 3) => {
                    let r = scalar("radius", 0.5);
                    Vec3::new(
                        r,
                        scalar("height", 1.0) * 0.5 + if shape == 2 { r } else { 0.0 },
                        r,
                    )
                }
                _ => {
                    notes.push(format!(
                        "entity '{}': a convex or mesh collider is baked from its model only",
                        e.values.first().and_then(|v| v.as_str()).unwrap_or("")
                    ));
                    continue;
                }
            };
            let offset = vector(numbers(Some(c), "offset"), Vec3::ZERO);
            out.extend(
                og_ai::nav::bake::box_triangles(offset, half)
                    .into_iter()
                    .map(|t| t.map(|v| m.transform_point3(v))),
            );
        }
        scene_triangles(block, m, models, dependencies, out, notes)?;
    }
    Ok(())
}

/// Sample the renderer's diagonals and holes, without a quantized reload.
fn terrain_triangles(
    source: &Source,
    terrain: &input::NavTerrain,
    bounds: Option<(Vec3, Vec3)>,
    out: &mut Vec<[Vec3; 3]>,
) -> Result<usize, String> {
    let field = super::terrain::get(source, &terrain.terrain.id)?.surface()?;
    let position = vector(terrain.position, Vec3::ZERO);
    let (cols, rows) = (field.columns as usize, field.rows as usize);
    // A stride beyond the grid is equivalent to its extent; cap before
    // adding it to an index so even a finite f32::MAX cannot overflow.
    let step = (terrain.step.unwrap_or(1.0).max(1.0) as usize).min(cols.max(rows));
    let cut = |c: usize, r: usize| {
        let Some(h) = &field.holes else {
            return false;
        };
        let (c1, r1) = ((c + step).min(cols - 1), (r + step).min(rows - 1));
        let n = (r..r1)
            .flat_map(|r| (c..c1).map(move |c| (c, r)))
            .filter(|&(c, r)| h[r * (cols - 1) + c] != 0)
            .count();
        n * 2 > (c1 - c) * (r1 - r)
    };
    let size = field.size;
    let cell = (size[0] / (cols - 1) as f32, size[1] / (rows - 1) as f32);
    let height = |c: usize, r: usize| field.heights[r.min(rows - 1) * cols + c.min(cols - 1)];
    let at = |c: usize, r: usize| {
        let (c, r) = (c.min(cols - 1), r.min(rows - 1));
        Vec3::new(
            position.x - size[0] * 0.5 + c as f32 * cell.0,
            position.y + height(c, r),
            position.z - size[1] * 0.5 + r as f32 * cell.1,
        )
    };
    let before = out.len();
    for r in (0..rows - 1).step_by(step) {
        for c in (0..cols - 1).step_by(step) {
            let (p00, p10) = (at(c, r), at(c + step, r));
            let (p01, p11) = (at(c, r + step), at(c + step, r + step));
            let inside = bounds.is_none_or(|(lo, hi)| {
                p11.x >= lo.x && p00.x <= hi.x && p11.z >= lo.z && p00.z <= hi.z
            });
            if inside && !cut(c, r) {
                if og_core::math::heightfield_zigzag(
                    height(c, r),
                    height(c + step, r),
                    height(c, r + step),
                    height(c + step, r + step),
                ) {
                    out.extend([[p00, p01, p11], [p00, p11, p10]]);
                } else {
                    out.extend([[p00, p01, p10], [p01, p11, p10]]);
                }
            }
        }
    }
    Ok(out.len() - before)
}

fn areas(input: &[input::Area]) -> Vec<(og_ai::nav::Shape, u8)> {
    input
        .iter()
        .map(|area| {
            use input::AreaShape::*;
            use og_ai::nav::Shape;
            let shape = match &area.shape {
                Box { center, size, yaw } => {
                    let center = vector(*center, Vec3::ZERO);
                    let half = vector(*size, Vec3::ONE) * 0.5;
                    match yaw.unwrap_or(0.0) {
                        0.0 => Shape::Box {
                            min: center - half,
                            max: center + half,
                        },
                        yaw => Shape::Oriented { center, half, yaw },
                    }
                }
                Cylinder {
                    center,
                    radius,
                    height,
                } => Shape::Cylinder {
                    base: vector(*center, Vec3::ZERO),
                    radius: radius.max(0.0),
                    height: height.unwrap_or(2.0).max(0.0),
                },
                Convex {
                    points,
                    bottom,
                    top,
                } => Shape::Convex {
                    points: points
                        .iter()
                        .copied()
                        .map(og_core::glam::Vec2::from_array)
                        .collect(),
                    bottom: bottom.unwrap_or(-1.0),
                    top: top.unwrap_or(2.0),
                },
            };
            (shape, area.id)
        })
        .collect()
}

fn links(input: &[input::Link]) -> Result<Vec<(u32, og_ai::nav::Link)>, CookError> {
    let mut ids = BTreeSet::new();
    input
        .iter()
        .map(|link| {
            if !ids.insert(link.id) {
                return Err(CookError(format!("link id {} is used twice", link.id)));
            }
            Ok((
                link.id,
                og_ai::nav::Link {
                    start: Vec3::from_array(link.start),
                    end: Vec3::from_array(link.end),
                    kind: match link.kind {
                        None | Some(input::LinkKind::Jump) => og_ai::nav::LinkKind::Jump,
                        Some(input::LinkKind::Ladder) => og_ai::nav::LinkKind::Ladder,
                        Some(input::LinkKind::Door) => og_ai::nav::LinkKind::Door,
                        Some(input::LinkKind::Teleport) => og_ai::nav::LinkKind::Teleport,
                    },
                    bidirectional: link.bidirectional.unwrap_or(true),
                    cost: link.cost.unwrap_or(0.0).max(0.0),
                    duration: link.duration.unwrap_or(0.0).max(0.0),
                    open: link.open.unwrap_or(true),
                    game: matches!(link.traverse, Some(input::Traverse::Game)),
                },
            ))
        })
        .collect()
}

pub(crate) fn cook_navmesh(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let prepared = s.navmesh.as_ref().expect("bound typed navigation");
    let input = &prepared.input;
    let areas = areas(&input.areas);
    let links = links(&input.links)?;
    let mut profiles = Vec::new();
    let mut names = BTreeSet::new();
    for agent in &input.agents {
        let name = agent.name.as_deref().unwrap_or("");
        if !names.insert(name) {
            return Err(CookError(format!("agent profile '{name}' is used twice")));
        }
        profiles.push((name, Some(agent)));
    }
    if let Some(i) = profiles.iter().position(|(n, _)| n.is_empty()) {
        let unnamed = profiles.remove(i);
        profiles.insert(0, unnamed);
    }
    if profiles.is_empty() {
        profiles.push(("", None));
    }
    let bounds = input.bounds.as_ref().map(|b| {
        (
            vector(b.min, Vec3::splat(-10.0)),
            vector(b.max, Vec3::splat(10.0)),
        )
    });
    let mut tris = prepared.triangles.clone();
    let mut notes = prepared.notes.clone();
    for terrain in &input.terrains {
        let n = terrain_triangles(s, terrain, bounds, &mut tris).map_err(CookError)?;
        notes.push(format!("terrain '{}': {n} triangles", terrain.terrain.id));
    }
    if tris.iter().flatten().any(|v| !v.is_finite()) {
        return Err(CookError("placed navigation geometry is not finite".into()));
    }
    let mut baked = Vec::with_capacity(profiles.len());
    for (name, agent) in &profiles {
        let params = og_ai::nav::NavParams {
            cell_size: input.cell_size.unwrap_or(0.2),
            cell_height: input.cell_height.unwrap_or(0.1),
            agent_radius: agent.and_then(|a| a.radius).unwrap_or(0.4),
            agent_height: agent.and_then(|a| a.height).unwrap_or(1.8),
            agent_climb: agent.and_then(|a| a.climb).unwrap_or(0.4),
            max_slope: agent.and_then(|a| a.slope).unwrap_or(45.0),
            tile_size: input.tile_size.unwrap_or(32.0).clamp(8.0, 256.0) as u32,
            max_error: input.max_error.unwrap_or(1.3),
        };
        let field = og_ai::nav::bake::bake(&tris, &params, bounds).map_err(CookError)?;
        let mut own = Vec::new();
        let (w, polys, tiles) = navmesh_blob(&field, &params, &areas, &links, stamp, &mut own)?;
        baked.push((*name, field, w, polys, tiles, own));
    }
    let (_, field, mut w, polys, tiles, own) = baked.remove(0);
    notes.extend(own);
    if profiles.len() > 1 || !profiles[0].0.is_empty() {
        let mut names = profiles[0].0.to_string();
        for (name, _, b, polys, tiles, own) in baked {
            notes.push(format!(
                "agent profile '{name}': {polys} polygons in {tiles} tiles"
            ));
            notes.extend(
                own.into_iter()
                    .map(|n| format!("agent profile '{name}': {n}")),
            );
            w.raw(&format!("profile.{name}"), b.finish(), 16);
            names.push('\n');
            names.push_str(name);
        }
        w.raw("profiles", names.into_bytes(), 1);
    }
    notes.insert(
        0,
        format!(
            "navmesh: {} triangles into {} × {} cells, {polys} polygons in {tiles} tiles",
            tris.len(),
            field.width,
            field.depth
        ),
    );
    Ok((w.finish(), notes))
}

/// The existing native blob writer; profile, area and link layouts are unchanged.
fn navmesh_blob(
    field: &og_ai::nav::Heightfield,
    params: &og_ai::nav::NavParams,
    areas: &[(og_ai::nav::Shape, u8)],
    links: &[(u32, og_ai::nav::Link)],
    stamp: u64,
    notes: &mut Vec<String>,
) -> Result<(BlobWriter, usize, usize), CookError> {
    let mesh = og_ai::nav::NavMesh::build_with_areas(field.clone(), areas.to_vec());
    let meta = format::NavMeshMeta {
        cell_size: params.cell_size,
        cell_height: params.cell_height,
        agent_height: params.agent_height,
        agent_radius: params.agent_radius,
        agent_climb: params.agent_climb,
        max_slope: params.max_slope,
        tile_size: params.tile_size,
        max_error: params.max_error,
        origin_x: field.origin.x,
        origin_y: field.origin.y,
        origin_z: field.origin.z,
        width: field.width,
        depth: field.depth,
        tiles: mesh.tiles.len() as u32,
    };
    // The heightfield as two meshopt vertex streams. A column's first span
    // is where the previous column's end, so only its count is written.
    let stream = |rows: Vec<u8>, stride: usize| -> Result<Vec<u8>, CookError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        super::model::encode_vertices(&rows, rows.len() / stride, stride)
            .map_err(|e| CookError(format!("navmesh heightfield: {e}")))
    };
    let mut next = 0;
    for &[first, count] in &field.columns {
        if first != next {
            return Err(CookError(
                "navmesh heightfield: a column's spans do not follow the previous column's".into(),
            ));
        }
        next += count;
    }
    let columns = stream(
        field
            .columns
            .iter()
            .flat_map(|c| c[1].to_le_bytes())
            .collect(),
        size_of::<format::NavColumnRow>(),
    )?;
    let spans = stream(
        field
            .spans
            .iter()
            .flat_map(|s| [s.floor.to_le_bytes(), s.clear.to_le_bytes()].concat())
            .collect(),
        size_of::<format::NavSpanRow>(),
    )?;
    let (mut tiles, mut verts, mut polys, mut indices) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for t in &mesh.tiles {
        tiles.push(format::NavTileRow {
            vert_first: verts.len() as u32,
            vert_count: t.verts.len() as u32,
            poly_first: polys.len() as u32,
            poly_count: t.polys.len() as u32,
        });
        for (v, l) in t.verts.iter().zip(&t.lattice) {
            verts.push(format::NavVertRow {
                x: v.x,
                y: v.y,
                z: v.z,
                lx: l.0,
                lz: l.1,
            });
        }
        for p in &t.polys {
            polys.push(format::NavPolyRow {
                index_first: indices.len() as u32,
                index_count: p.verts.len() as u32,
                region: p.region,
            });
            indices.extend(p.verts.iter().map(|&i| i as u32));
        }
    }
    let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let mut w = BlobWriter::new(
        format::NavMesh::type_hash(),
        format::NavMesh::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], format::row_hash::<format::NavMeshMeta>())
        .raw("columns", columns, 16)
        .raw("spans", spans, 16)
        .table("tiles", &tiles, format::row_hash::<format::NavTileRow>())
        .table("verts", &verts, format::row_hash::<format::NavVertRow>())
        .table("polys", &polys, format::row_hash::<format::NavPolyRow>())
        .raw("poly_verts", index_bytes, 16);
    if !areas.is_empty() {
        let poly_areas: Vec<u8> = mesh
            .tiles
            .iter()
            .flat_map(|t| t.polys.iter().map(|p| p.area))
            .collect();
        let mut shapes = Vec::new();
        for (shape, area) in areas {
            shapes.push(*area);
            shape.write(&mut shapes);
        }
        w.raw("poly_areas", poly_areas, 1).raw("areas", shapes, 4);
        notes.push(format!(
            "areas: {} shapes, {} polygons off the ground area",
            areas.len(),
            mesh.tiles
                .iter()
                .flat_map(|t| &t.polys)
                .filter(|p| p.area != 0)
                .count()
        ));
    }
    if !links.is_empty() {
        let linked = og_ai::nav::NavMesh::from_tiles(field.clone(), mesh.tiles.clone())
            .map(|m| m.with_links(links.to_vec()));
        let mut bytes = Vec::new();
        for (id, l) in links {
            bytes.extend_from_slice(&id.to_le_bytes());
            l.write(&mut bytes);
        }
        w.raw("links", bytes, 4);
        let off: Vec<String> = links
            .iter()
            .filter(|(id, _)| linked.as_ref().is_none_or(|m| m.link_ends(*id).is_none()))
            .map(|(id, _)| id.to_string())
            .collect();
        notes.push(format!("links: {}", links.len()));
        if !off.is_empty() {
            notes.push(format!("warning: links {} have an end more than 1 m from the mesh; agents never cross them", off.join(", ")));
        }
    }
    Ok((w, mesh.poly_count(), mesh.tiles.len()))
}

pub(crate) static NAVMESH: KindDef = KindDef {
    name: "navmesh",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_nav_mesh,
};
pub(crate) static KINDS: &[&KindDef] = &[&NAVMESH];

fn plan_nav_mesh<'a>(
    _: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let prepared = s.navmesh.as_ref().expect("bound typed navigation");
    state
        .parsed_refs
        .entry(s.rel.clone())
        .or_default()
        .extend(prepared.dependencies.iter().cloned());
    let st = stamp(
        &[Area::Nav],
        &[b"typed navmesh 1", &hash, &prepared.hash.to_le_bytes()],
    );
    outs.push((
        s.rel.clone(),
        format::NavMesh::NAME,
        st,
        Box::new(NavMeshJob(s)),
    ));
}
struct NavMeshJob<'a>(&'a Source);
impl Job for NavMeshJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_navmesh(self.0, p.stamp).map_err(|e| {
            self.0
                .navmesh
                .as_ref()
                .expect("bound typed navigation")
                .error(e.0)
        })
    }
}
