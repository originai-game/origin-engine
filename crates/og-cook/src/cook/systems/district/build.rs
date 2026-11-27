//! A district built: its plan, its buildings and streets as meshes by
//! material, and the entities that place them, cell by cell.

use std::collections::{BTreeMap, BTreeSet};

use glam::{DQuat, DVec2, DVec3};
use og_core::hash::{hash_bytes, splitmix64, splitmix64_next};
use og_shape::generators::Mesh;
use og_shape::generators::earcut::earcut;
use og_shape::generators::grammar::{self, Grammar};
use og_shape::generators::layout::{self, Layout, LayoutDesc, Poly, Region};

use super::parse::{District, Material, PropCollider};
use super::{glb, parse, texts};

/// What a district expands to: its scene, its cells' scenes and its
/// models, by asset id.
pub(super) struct Output {
    pub scene: String,
    pub scenes: Vec<(String, String)>,
    pub models: Vec<(String, Vec<u8>)>,
    pub carve: Option<(String, og_core::ogt::Entry)>,
}

/// Triangles by material and detail level.
pub(super) type Acc = BTreeMap<(String, u32), Mesh>;

/// What one cell holds (the whole district without cells).
#[derive(Default)]
struct Cell {
    /// Its models by the suffix of their asset id ("" the cell's own).
    models: BTreeMap<String, Acc>,
    entities: Vec<og_core::ogt::Entry>,
    top: f64,
}

/// Draws from a seed.
struct Draw(u64);

impl Draw {
    fn new(seed: u64, what: &str) -> Draw {
        Draw(splitmix64(seed ^ hash_bytes(what.as_bytes())))
    }

    fn unit(&mut self) -> f64 {
        (splitmix64_next(&mut self.0) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// An index by weight.
    fn pick(&mut self, weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        let mut at = self.unit() * total;
        for (i, w) in weights.iter().enumerate() {
            at -= w;
            if at < 0.0 {
                return i;
            }
        }
        weights.len() - 1
    }
}

/// Each label's materials, and what a building drew for it.
struct Library<'a> {
    materials: &'a [Material],
    by_label: BTreeMap<&'a str, Vec<usize>>,
}

impl<'a> Library<'a> {
    fn new(materials: &'a [Material]) -> Library<'a> {
        let mut by_label: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (i, m) in materials.iter().enumerate() {
            for l in &m.labels {
                by_label.entry(l.as_str()).or_default().push(i);
            }
        }
        Library {
            materials,
            by_label,
        }
    }

    /// The material `seed` draws for `label`, with its colour and phase.
    fn draw(&self, label: &str, seed: u64, whose: &str) -> Result<Look, String> {
        let Some(options) = self.by_label.get(label) else {
            return Err(format!(
                "{whose} makes parts labelled `{label}`, which no material of the library covers; it covers {}",
                self.by_label.keys().copied().collect::<Vec<_>>().join(", ")
            ));
        };
        let weights: Vec<f64> = options.iter().map(|&i| self.materials[i].weight).collect();
        // Keyed by the materials it chooses among, not the label: labels
        // the same materials cover (a wall and its window jambs) draw the
        // same one, so a building's brick runs into its reveals.
        let among: String = options
            .iter()
            .map(|&i| self.materials[i].name.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let mut d = Draw::new(seed, &among);
        let m = &self.materials[options[d.pick(&weights)]];
        let mut d = Draw::new(seed, &m.name);
        let value = 1.0 + m.value * (2.0 * d.unit() - 1.0);
        let warmth = m.warmth * (2.0 * d.unit() - 1.0);
        let color =
            [value * (1.0 + warmth), value, value * (1.0 - warmth)].map(|c| c.max(0.0) as f32);
        Ok(Look {
            material: m.name.clone(),
            tile: DVec2::new(m.tile[0], if m.metric_height { 1.0 } else { m.tile[1] }),
            // Grammar v is negative height: a facade's bottom row is
            // at ground level (v=1), never slid to a random storey.
            phase: DVec2::new(
                d.unit(),
                if m.metric_height {
                    0.0
                } else if m.facade {
                    1.0
                } else {
                    d.unit()
                },
            ),
            color: [color[0], color[1], color[2], 1.0],
        })
    }
}

/// A part's material as one building draws it.
struct Look {
    material: String,
    tile: DVec2,
    phase: DVec2,
    color: [f32; 4],
}

/// Appends a mesh whose uvs are in metres, as `look` draws it, moved by
/// `-origin`.
fn add_mesh(acc: &mut Acc, look: &Look, detail: u32, m: &Mesh, origin: DVec3) {
    let out = acc.entry((look.material.clone(), detail)).or_default();
    let first = out.positions.len() as u32;
    let o = origin.as_vec3();
    for (i, p) in m.positions.iter().enumerate() {
        out.positions.push([p[0] - o.x, p[1] - o.y, p[2] - o.z]);
        out.normals.push(m.normals[i]);
        let uv = m.uvs[i];
        out.uvs.push([
            (f64::from(uv[0]) / look.tile.x + look.phase.x) as f32,
            (f64::from(uv[1]) / look.tile.y + look.phase.y) as f32,
        ]);
        out.colors.push(look.color);
    }
    out.indices.extend(m.indices.iter().map(|i| first + i));
}

/// A planar polygon (with at most one hole) facing `n`, its uvs `uv` of
/// each corner in metres.
fn polygon(outer: &[DVec3], hole: &[DVec3], n: DVec3, uv: &dyn Fn(DVec3) -> DVec2) -> Mesh {
    let pts: Vec<DVec3> = outer.iter().chain(hole).copied().collect();
    let (a, b) = if n.y.abs() >= n.x.abs().max(n.z.abs()) {
        (0, 2)
    } else if n.x.abs() >= n.z.abs() {
        (1, 2)
    } else {
        (0, 1)
    };
    let flat: Vec<f64> = pts.iter().flat_map(|p| [p[a], p[b]]).collect();
    let holes = if hole.is_empty() {
        Vec::new()
    } else {
        vec![outer.len()]
    };
    let tris = earcut(&flat, &holes);
    let mut m = Mesh::default();
    for p in &pts {
        m.positions.push(p.as_vec3().to_array());
        m.normals.push(n.as_vec3().to_array());
        let t = uv(*p);
        m.uvs.push([t.x as f32, t.y as f32]);
    }
    for t in tris.as_chunks::<3>().0 {
        let [i, j, k] = *t;
        let face = (pts[j] - pts[i]).cross(pts[k] - pts[i]);
        let t = if face.dot(n) >= 0.0 {
            [i, j, k]
        } else {
            [i, k, j]
        };
        m.indices.extend(t.map(|x| x as u32));
    }
    m
}

fn at(p: DVec2, y: f64) -> DVec3 {
    DVec3::new(p.x, y, p.y)
}

/// A level face's uvs: the world's x and z, in metres.
fn level(p: DVec3) -> DVec2 {
    DVec2::new(p.x, -p.z)
}

/// The road's ground: the region, or the roadway along the street a
/// region lines.
pub(super) fn roads(d: &District, plan: &Layout) -> Vec<Vec<DVec2>> {
    match &d.region {
        Region::Area(points) => vec![points.clone()],
        Region::Along { .. } => {
            let s = plan.streets.last().expect("the street it lines");
            let h = s.width / 2.0 + s.sidewalk;
            let p = &s.points;
            let dir = |i: usize| (p[i + 1] - p[i]).normalize();
            let mitre = |i: usize| -> DVec2 {
                if i == 0 || i == p.len() - 1 {
                    return dir(i.min(p.len() - 2)).perp();
                }
                let (a, b) = (dir(i - 1).perp(), dir(i).perp());
                let m = (a + b).normalize_or_zero();
                if m.dot(a) < 0.1 { a } else { m / m.dot(a) }
            };
            (0..p.len() - 1)
                .map(|i| {
                    let (ma, mb) = (mitre(i), mitre(i + 1));
                    vec![
                        p[i] - ma * h,
                        p[i + 1] - mb * h,
                        p[i + 1] + mb * h,
                        p[i] + ma * h,
                    ]
                })
                .collect()
        }
    }
}

/// A turn about +y by `a` radians, by og-core's own sine so the text is
/// the same on every host.
fn yaw(a: f64) -> DQuat {
    let (s, c) = og_core::math::double::sin_cos(a / 2.0);
    DQuat::from_xyzw(0.0, s, 0.0, c)
}

/// The yaw that turns +x to `e` (in x and z).
fn yaw_x(e: DVec2) -> DQuat {
    yaw(og_core::math::double::atan2(-e.y, e.x))
}

/// The yaw that turns +z (a model's front) to `f` (in x and z).
fn yaw_z(f: DVec2) -> DQuat {
    yaw(og_core::math::double::atan2(f.x, f.y))
}

/// Builds district `d` (at asset id `rel`), reading what it names by
/// `read`.
pub(super) fn build(
    rel: &str,
    d: &District,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    prepared: &BTreeMap<String, Grammar>,
) -> Result<Output, String> {
    let text = |id: &str| read(id).map(|b| String::from_utf8_lossy(&b).into_owned());
    let materials =
        parse::materials(&text(&d.materials)?).map_err(|e| format!("{}: {e}", d.materials))?;
    let library = Library::new(&materials);
    let mut grammars: BTreeMap<&str, &Grammar> = BTreeMap::new();
    for r in &d.buildings.rules {
        let g = prepared
            .get(&r.grammar)
            .ok_or_else(|| format!("{}: not a checked grammar content ID", r.grammar))?;
        grammars.insert(&r.grammar, g);
    }
    let plan = layout::layout(&LayoutDesc {
        seed: d.seed,
        region: d.region.clone(),
        grid: d.grid.clone(),
        streets: d.streets.clone(),
        lots: d.lots.clone(),
        props: d.props.iter().map(|p| p.desc.clone()).collect(),
    })?;
    for r in &d.buildings.rules {
        if let Some(n) = &r.street
            && !plan.streets.iter().any(|s| &s.name == n)
        {
            return Err(format!(
                "rule `{}` names street `{n}`, which the district does not have",
                r.grammar
            ));
        }
    }
    // Cells over the plan's ground: (i, j) from its least corner.
    let ground: Vec<DVec2> = roads(d, &plan).into_iter().flatten().collect();
    let lo = ground
        .iter()
        .fold(DVec2::splat(f64::INFINITY), |a, p| a.min(*p));
    let size = d.cells.as_ref().map(|c| c.size);
    let cell_of = |p: DVec2| -> (i64, i64) {
        size.map_or((0, 0), |s| {
            (
                ((p.x - lo.x) / s).floor() as i64,
                ((p.y - lo.y) / s).floor() as i64,
            )
        })
    };
    let centre = |c: (i64, i64)| -> DVec3 {
        size.map_or(DVec3::ZERO, |s| {
            DVec3::new(
                lo.x + (c.0 as f64 + 0.5) * s,
                0.0,
                lo.y + (c.1 as f64 + 0.5) * s,
            )
        })
    };
    let mut cells: BTreeMap<(i64, i64), Cell> = BTreeMap::new();
    let kerb = d.kerb;
    let street_look = |label: &str| library.draw(label, d.seed, "the streets");

    // The roads, cut along the cells.
    for road in roads(d, &plan) {
        let pieces: Vec<((i64, i64), Vec<DVec2>)> = match size {
            None => vec![((0, 0), road)],
            Some(s) => {
                let (rlo, rhi) = road.iter().fold(
                    (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
                    |(a, b), p| (a.min(*p), b.max(*p)),
                );
                let (c0, c1) = (cell_of(rlo), cell_of(rhi));
                let mut out = Vec::new();
                for i in c0.0..=c1.0 {
                    for j in c0.1..=c1.1 {
                        let x0 = lo.x + i as f64 * s;
                        let z0 = lo.y + j as f64 * s;
                        let p = Poly::new(road.clone(), None)
                            .clip(DVec2::X, x0 + s, None)
                            .clip(-DVec2::X, -x0, None)
                            .clip(DVec2::Y, z0 + s, None)
                            .clip(-DVec2::Y, -z0, None)
                            .tidy();
                        if p.len() >= 3 && p.area().abs() > 1e-9 {
                            out.push(((i, j), p.points));
                        }
                    }
                }
                out
            }
        };
        for (c, poly) in pieces {
            let look = street_look("road")?;
            let pts: Vec<DVec3> = poly.iter().map(|p| at(*p, 0.0)).collect();
            let m = polygon(&pts, &[], DVec3::Y, &level);
            add_mesh(
                cells
                    .entry(c)
                    .or_default()
                    .models
                    .entry(String::new())
                    .or_default(),
                &look,
                0,
                &m,
                centre(c),
            );
        }
    }

    // Each block's pavement, its ground and its kerbs.
    for block in &plan.blocks {
        let mid = block.poly.points.iter().copied().sum::<DVec2>() / block.poly.len() as f64;
        let c = cell_of(mid);
        let acc = cells
            .entry(c)
            .or_default()
            .models
            .entry(String::new())
            .or_default();
        let pave: Vec<DVec3> = block.pavement.iter().map(|p| at(*p, kerb)).collect();
        let own: Vec<DVec3> = block.poly.points.iter().map(|p| at(*p, kerb)).collect();
        if block.pavement != block.poly.points {
            add_mesh(
                acc,
                &street_look("pavement")?,
                0,
                &polygon(&pave, &own, DVec3::Y, &level),
                centre(c),
            );
        }
        add_mesh(
            acc,
            &street_look("ground")?,
            0,
            &polygon(&own, &[], DVec3::Y, &level),
            centre(c),
        );
        if kerb > 0.0 {
            let look = street_look("kerb")?;
            let n = pave.len();
            for i in 0..n {
                let (p, q) = (block.pavement[i], block.pavement[(i + 1) % n]);
                let e = (q - p).normalize();
                let out = DVec3::new(e.y, 0.0, -e.x);
                let quad = [at(p, 0.0), at(q, 0.0), at(q, kerb), at(p, kerb)];
                let len = p.distance(q);
                let m = polygon(&quad, &[], out, &|x: DVec3| {
                    DVec2::new((x - quad[0]).dot(at(e, 0.0)) % (len + 1.0), -x.y)
                });
                add_mesh(acc, &look, 0, &m, centre(c));
            }
        }
    }

    // Each lot's building, or none.
    let b = &d.buildings;
    let mut inserts = 0usize;
    for (k, lot) in plan.lots.iter().enumerate() {
        let mut draw = Draw::new(lot.seed, "building");
        if b.rules.is_empty() || draw.unit() >= b.density {
            continue;
        }
        let street = &plan.streets[lot.street].name;
        let fits: Vec<&parse::RuleUse> = b
            .rules
            .iter()
            .filter(|r| r.street.as_ref().is_none_or(|s| s == street))
            .collect();
        if fits.is_empty() {
            continue;
        }
        let rule = fits[draw.pick(&fits.iter().map(|r| r.weight).collect::<Vec<_>>())];
        let g = &grammars[rule.grammar.as_str()];
        let mid = lot.footprint.iter().copied().sum::<DVec2>() / lot.footprint.len() as f64;
        // Noise over the town (tall streets and low ones) mixed with the
        // lot's own draw, by the share the district states.
        let [freq, share] = b.noise;
        let n =
            og_core::noise::perlin2(d.seed as u32, (mid.x * freq) as f32, (mid.y * freq) as f32);
        let n = (0.5 + 0.5 * f64::from(n)).clamp(0.0, 1.0);
        let t = n * share + draw.unit() * (1.0 - share);
        let height = b.height[0] + (b.height[1] - b.height[0]) * t;
        let params: BTreeMap<String, f64> =
            if g.params.contains_key("height") && (b.height[0] > 0.0 || b.height[1] > 0.0) {
                BTreeMap::from([("height".into(), height)])
            } else {
                BTreeMap::new()
            };
        let built = grammar::build(
            g,
            &grammar::Lot {
                footprint: lot.footprint.clone(),
                base: kerb,
                seed: lot.seed,
            },
            &params,
        )
        .map_err(|e| format!("{}: lot {k}: {e}", rule.grammar))?;
        let c = cell_of(mid);
        let o = centre(c);
        let cell = cells.entry(c).or_default();
        let key = if d.merge {
            String::new()
        } else {
            format!("b{k}")
        };
        let whose = format!("grammar `{}`", rule.grammar);
        let mut top = kerb;
        for part in &built.parts {
            let look = library.draw(&part.label, lot.seed, &whose)?;
            top = part
                .mesh
                .positions
                .iter()
                .fold(top, |t, p| t.max(f64::from(p[1])));
            add_mesh(
                cell.models.entry(key.clone()).or_default(),
                &look,
                part.detail,
                &part.mesh,
                o,
            );
        }
        cell.top = cell.top.max(top);
        for ins in &built.inserts {
            let q = DQuat::from_mat3(&glam::DMat3::from_cols(ins.x, ins.y, ins.z));
            let p = ins.origin + ins.x * (ins.size.x / 2.0) - o;
            cell.entities.push(texts::placed(
                &format!("{rel}/insert-{inserts}"),
                &ins.asset,
                p,
                q,
                None,
            ));
            inserts += 1;
        }
        // It collides as the box round its lot, as tall as it stands.
        let (e, n) = {
            let e = (lot.footprint[1] - lot.footprint[0]).normalize();
            (e, e.perp())
        };
        let span = |a: DVec2| {
            lot.footprint
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), p| {
                    let x = (*p - lot.footprint[0]).dot(a);
                    (l.min(x), h.max(x))
                })
        };
        let (ue, un) = (span(e), span(n));
        let foot = lot.footprint[0] + e * ((ue.0 + ue.1) / 2.0) + n * ((un.0 + un.1) / 2.0);
        let size = DVec3::new(ue.1 - ue.0, top - kerb, un.1 - un.0);
        let p = DVec3::new(foot.x, kerb + size.y / 2.0, foot.y) - o;
        cell.entities.push(texts::collider(
            &format!("{rel}/building-{k}"),
            p,
            yaw_x(e),
            &PropCollider::Box(size.to_array()),
        ));
    }

    // Each prop where the plan stands it.
    for (n, s) in plan.sites.iter().enumerate() {
        let prop = &d.props[s.prop];
        let c = cell_of(s.position);
        let p = at(s.position, kerb) - centre(c);
        let cell = cells.entry(c).or_default();
        let name = format!("{rel}/{}-{n}", prop.desc.name);
        cell.entities.push(texts::placed(
            &name,
            &prop.model,
            p,
            yaw_z(s.facing),
            prop.collider.as_ref(),
        ));
    }

    // The models, the cells' scenes and the district's.
    let dir_depth = rel.matches('/').count();
    let uri = |id: &str| format!("{}{id}", "../".repeat(dir_depth));
    let mut out = Output {
        scene: String::new(),
        scenes: Vec::new(),
        models: Vec::new(),
        carve: d
            .terrain
            .as_ref()
            .map(|(id, depth)| (id.clone(), super::terrain::carve(d, &plan, *depth))),
    };
    let mut placed = Vec::new();
    let group = format!("district-{:08x}", hash_bytes(rel.as_bytes()) as u32);
    for (c, cell) in &cells {
        let base = match size {
            None => rel.to_string(),
            Some(_) => format!("{rel}.cell-{}-{}", c.0, c.1),
        };
        let mut entities = Vec::new();
        let mut ids = Vec::new();
        for (suffix, acc) in &cell.models {
            let id = if suffix.is_empty() {
                format!("{base}.glb")
            } else {
                format!("{base}.{suffix}.glb")
            };
            let used: BTreeSet<&str> = acc.keys().map(|(m, _)| m.as_str()).collect();
            let mats: Vec<&Material> = materials
                .iter()
                .filter(|m| used.contains(m.name.as_str()))
                .collect();
            out.models.push((id.clone(), glb::write(acc, &mats, &uri)?));
            // Named as the model is: names are unique in the whole
            // world, across cells and districts.
            entities.push(texts::placed(
                &format!("{}/model", id.trim_end_matches(".glb")),
                &id,
                DVec3::ZERO,
                DQuat::IDENTITY,
                None,
            ));
            ids.push(id);
        }
        entities.extend(cell.entities.iter().cloned());
        match (&d.cells, size) {
            (Some(cells_desc), Some(s)) => {
                let scene_id = format!("{base}.ogt");
                let stream = format!("{group}-{}-{}", c.0, c.1);
                out.scenes.push((
                    scene_id.clone(),
                    texts::scene(&scene_id, d.hlod.as_ref(), Some((&stream, &ids)), &entities)?,
                ));
                placed.push(texts::cell(
                    &base,
                    &scene_id,
                    centre(*c),
                    s,
                    cell.top + 10.0,
                    cells_desc.load,
                ));
            }
            _ => placed.extend(entities),
        }
    }
    let hlod = if d.cells.is_some() {
        None
    } else {
        d.hlod.as_ref()
    };
    out.scene = texts::scene(rel, hlod, None, &placed)?;
    Ok(out)
}
