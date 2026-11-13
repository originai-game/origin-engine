//! Mesh conformance geometry, face budgets and outline hulls.

use super::*;

// ── geometry ─────────────────────────────────────────────────────────────

/// What conforming one primitive did, for the model's report.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Changes {
    /// Triangles before and after.
    pub faces: (usize, usize),
    /// Set by the conformance: normals (so the cook keeps them), vertex
    /// colours (so the primitive stores them).
    pub normals: bool,
    pub colors: bool,
    /// Voxelised: the edge used and the voxels.
    pub voxel: Option<(f32, usize)>,
    /// Bevelled: hard edges and triangles added.
    pub bevel: Option<(usize, usize)>,
    /// Distinct vertex colours after baking.
    pub distinct: usize,
    /// Why a step was left out.
    pub skipped: Vec<String>,
}

pub(super) fn pos(v: &FVertex) -> Vec3 {
    Vec3::from(v.p)
}

pub(super) fn cross(v: &[FVertex], t: &[u32]) -> Vec3 {
    let [a, b, c] = [0, 1, 2].map(|k| pos(&v[t[k] as usize]));
    (b - a).cross(c - a)
}

fn pkey(p: [f32; 3]) -> [u32; 3] {
    // -0.0 and 0.0 are one position.
    p.map(|x| (x + 0.0).to_bits())
}

/// Every corner its own vertex (sources follow).
fn unweld(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>) {
    *v = idx.iter().map(|&i| v[i as usize]).collect();
    *src = idx.iter().map(|&i| src[i as usize]).collect();
    *idx = (0..v.len() as u32).collect();
}

/// Only the vertices `idx` uses, in first-use order.
fn compact(v: &mut Vec<FVertex>, idx: &mut [u32], src: &mut Vec<u32>) {
    let mut at = vec![u32::MAX; v.len()];
    let (mut nv, mut ns) = (Vec::new(), Vec::new());
    for i in idx.iter_mut() {
        if at[*i as usize] == u32::MAX {
            at[*i as usize] = nv.len() as u32;
            nv.push(v[*i as usize]);
            ns.push(src[*i as usize]);
        }
        *i = at[*i as usize];
    }
    *v = nv;
    *src = ns;
}

fn adapter(v: &[FVertex]) -> meshopt::VertexDataAdapter<'_> {
    meshopt::VertexDataAdapter::new(bytemuck::cast_slice(v), std::mem::size_of::<FVertex>(), 0)
        .expect("positions lead the vertex")
}

/// `idx` over the first vertex at each position.
fn by_position(v: &[FVertex], idx: &[u32]) -> Vec<u32> {
    let mut first: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let rep: Vec<u32> = (0..v.len() as u32)
        .map(|i| *first.entry(pkey(v[i as usize].p)).or_insert(i))
        .collect();
    idx.iter().map(|&i| rep[i as usize]).collect()
}

/// Without `ArtShape.silhouette`, the most decimation may move a model's
/// surface: this fraction of its bounding radius (half its longest side).
pub const SHAPE_ERROR: f32 = 0.05;

/// `idx` simplified towards `target` triangles (0: as far as `error`
/// allows) with its topology kept: meshoptimizer's edge collapses, never
/// its sloppy simplifier, the open borders (a primitive's edges against
/// its neighbours of other materials) locked, the surface moved at most
/// `error` (in the vertices' units). First over the vertices, so every
/// attribute seam holds; then, when that stops above the target (a flat-
/// shaded or seamed mesh splits every corner), welded by position with
/// the uv seams of a `textured` mesh locked. The indices, and whether the
/// second pass made them (its corners' normals no longer belong to them).
fn simplified(
    v: &[FVertex],
    idx: &[u32],
    target: usize,
    error: f32,
    textured: bool,
) -> (Vec<u32>, bool) {
    let want = target * 3;
    let a = adapter(v);
    let opts = meshopt::SimplifyOptions::LockBorder | meshopt::SimplifyOptions::ErrorAbsolute;
    let out = meshopt::simplify(idx, &a, want, error, opts, None);
    if out.len() <= want {
        return (out, false);
    }
    let shadow = by_position(v, idx);
    // A position whose corners disagree on their uv is a seam.
    let mut uv_at: BTreeMap<[u32; 3], [u32; 2]> = BTreeMap::new();
    let mut seam = vec![false; v.len()];
    if textured {
        let mut split: BTreeSet<[u32; 3]> = BTreeSet::new();
        for &i in idx {
            let x = &v[i as usize];
            let uv = x.uv.map(f32::to_bits);
            if *uv_at.entry(pkey(x.p)).or_insert(uv) != uv {
                split.insert(pkey(x.p));
            }
        }
        for (i, x) in v.iter().enumerate() {
            seam[i] = split.contains(&pkey(x.p));
        }
    }
    let welded = meshopt::simplify_with_locks(&shadow, &a, &seam, want, error, opts, None);
    if welded.len() >= out.len() {
        return (out, false);
    }
    (reattach(v, idx, &shadow, &welded, &seam), true)
}

/// The corners of `welded` (over `shadow`, the first vertex at each
/// position) given back their own vertices: a triangle the source has
/// takes its corners; elsewhere a corner on a uv seam takes, of the
/// vertices at its position, the one whose uv is nearest the triangle's
/// other corners'.
fn reattach(v: &[FVertex], idx: &[u32], shadow: &[u32], welded: &[u32], seam: &[bool]) -> Vec<u32> {
    let mut source: BTreeMap<[u32; 3], [u32; 3]> = BTreeMap::new();
    let mut at: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (t, s) in idx.as_chunks::<3>().0.iter().zip(shadow.as_chunks::<3>().0) {
        for k in 0..3 {
            source
                .entry([s[k], s[(k + 1) % 3], s[(k + 2) % 3]])
                .or_insert([t[k], t[(k + 1) % 3], t[(k + 2) % 3]]);
            let list = at.entry(s[k]).or_default();
            if !list.contains(&t[k]) {
                list.push(t[k]);
            }
        }
    }
    let uv = |i: u32| glam::Vec2::from(v[i as usize].uv);
    let mut out = Vec::with_capacity(welded.len());
    for t in welded.as_chunks::<3>().0 {
        if let Some(o) = source.get(t) {
            out.extend(o);
            continue;
        }
        let plain: Vec<glam::Vec2> = t
            .iter()
            .filter(|&&r| !seam[r as usize])
            .map(|&r| uv(r))
            .collect();
        for &r in t {
            let pick = match (seam[r as usize], plain.is_empty()) {
                (true, false) => {
                    let mean = plain.iter().sum::<glam::Vec2>() / plain.len() as f32;
                    at[&r]
                        .iter()
                        .copied()
                        .min_by(|a, b| uv(*a).distance(mean).total_cmp(&uv(*b).distance(mean)))
                        .unwrap_or(r)
                }
                _ => r,
            };
            out.push(pick);
        }
    }
    out
}

/// Simplifies to at most `target` triangles where it can within `error`
/// ([`simplified`]); whether the corners' normals must be made again.
fn decimate(
    v: &mut Vec<FVertex>,
    idx: &mut Vec<u32>,
    src: &mut Vec<u32>,
    target: usize,
    error: f32,
    textured: bool,
) -> bool {
    if idx.len() / 3 <= target {
        return false;
    }
    let (mut out, welded) = simplified(v, idx, target, error, textured);
    if out.len() < 3 || out.len() >= idx.len() {
        return false;
    }
    compact(v, &mut out, src);
    *idx = out;
    welded
}

/// Which primitive a plan's target is for: its position and index
/// accessors (a primitive drawn from two meshes is one).
type PrimKey = (Option<usize>, Option<usize>);

fn prim_key(p: &gltf::Primitive) -> PrimKey {
    (
        p.get(&gltf::Semantic::Positions).map(|a| a.index()),
        p.indices().map(|a| a.index()),
    )
}

/// How a model meets its face budget, made once per model
/// (`GltfSource::conform_plan`): each primitive's target and the most its
/// surface may move.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    /// Triangles per primitive; none: the model is within its budget.
    targets: BTreeMap<PrimKey, usize>,
    /// The error bound in metres and what set it.
    error: (f32, String),
    /// The triangles drawn at the error bound, when that is past the
    /// budget: then each primitive stops there.
    over: Option<usize>,
}

fn textured(p: &gltf::Primitive) -> bool {
    let m = p.material();
    m.pbr_metallic_roughness().base_color_texture().is_some()
        || m.normal_texture().is_some()
        || m.emissive_texture().is_some()
        || m.occlusion_texture().is_some()
        || m.pbr_metallic_roughness()
            .metallic_roughness_texture()
            .is_some()
}

/// A primitive's error bound in its own units: `metres` over the scale of
/// the node that places it.
fn local_error(src: &GltfSource, p: &gltf::Primitive, metres: f32) -> f32 {
    let (s, _, _) = placement_of(src, p).to_scale_rotation_translation();
    let scale = s.abs().max_element();
    if scale > 0.0 { metres / scale } else { metres }
}

impl Plan {
    /// The budget shared over the model's primitives (each counted once
    /// per node drawing it): every primitive simplified as far as the
    /// error bound lets it (`floor`), then the budget's remainder handed
    /// out in proportion to what each has left, so the model lands at the
    /// budget, not below it; or, when even the floors are past it, at the
    /// floors.
    fn of(c: &Conform, src: &GltfSource) -> Plan {
        let metres = if c.silhouette > 0.0 {
            (c.silhouette, "ArtShape.silhouette".to_string())
        } else {
            (
                SHAPE_ERROR * extent(src) / 2.0,
                format!("{:.0} % of its bounding radius", SHAPE_ERROR * 100.0),
            )
        };
        let mut plan = Plan {
            error: metres.clone(),
            ..Plan::default()
        };
        let (budget, _) = c.budget(src);
        let drawn = drawn_triangles(src);
        if budget == 0 || drawn <= budget as usize {
            return plan;
        }
        let mut uses = vec![0usize; src.doc.meshes().count()];
        if let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) {
            let mut stack: Vec<gltf::Node> = scene.nodes().collect();
            while let Some(n) = stack.pop() {
                if let Some(m) = n.mesh() {
                    uses[m.index()] += 1;
                }
                stack.extend(n.children());
            }
        }
        // Per primitive: (times drawn, triangles, floor).
        let mut prims: BTreeMap<PrimKey, (usize, usize, usize)> = BTreeMap::new();
        for m in src.doc.meshes() {
            for p in m
                .primitives()
                .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
            {
                let key = prim_key(&p);
                if let Some(e) = prims.get_mut(&key) {
                    e.0 += uses[m.index()].max(1);
                    continue;
                }
                let reader = p.reader(|b| Some(&src.buffers[b.index()].0[..]));
                let Some(pos) = reader.read_positions().map(|x| x.collect::<Vec<_>>()) else {
                    continue;
                };
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|u| u.into_f32().collect())
                    .unwrap_or_default();
                let v: Vec<FVertex> = pos
                    .iter()
                    .enumerate()
                    .map(|(i, p)| FVertex {
                        p: *p,
                        n: [0.0, 0.0, 1.0],
                        uv: uvs.get(i).copied().unwrap_or([0.0; 2]),
                        c: [1.0; 4],
                        t: [0.0; 4],
                        key: [0, 0],
                    })
                    .collect();
                let mut idx: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..v.len() as u32).collect());
                idx.truncate(idx.len() / 3 * 3);
                if idx.iter().any(|&i| i as usize >= v.len()) {
                    continue;
                }
                let tris = idx.len() / 3;
                let (out, _) =
                    simplified(&v, &idx, 0, local_error(src, &p, metres.0), textured(&p));
                let floor = if out.len() >= 3 {
                    (out.len() / 3).min(tris)
                } else {
                    tris
                };
                prims.insert(key, (uses[m.index()].max(1), tris, floor));
            }
        }
        let floors: usize = prims.values().map(|(u, _, f)| u * f).sum();
        let full: usize = prims.values().map(|(u, t, _)| u * t).sum();
        let budget = budget as usize;
        if floors >= budget {
            plan.over = Some(floors);
        }
        for (k, (_, t, f)) in prims {
            let share = if floors >= budget || full <= floors {
                0
            } else {
                ((t - f) as u64 * (budget - floors) as u64 / (full - floors) as u64) as usize
            };
            plan.targets.insert(k, f + share);
        }
        plan
    }

    fn get<'a>(c: &Conform, src: &'a GltfSource) -> &'a Plan {
        src.conform_plan.get_or_init(|| Plan::of(c, src))
    }
}

/// Chamfers the hard edges by `radius`; answers (hard edges, triangles
/// added), or `None` without a hard edge.
pub(super) fn bevel(
    v: &mut Vec<FVertex>,
    idx: &mut Vec<u32>,
    src: &mut Vec<u32>,
    radius: f32,
    hard_angle: f32,
) -> Option<(usize, usize)> {
    let tris = idx.len() / 3;
    let shadow = by_position(v, idx);
    let id = |f: usize, k: usize| shadow[f * 3 + k % 3];
    let normal: Vec<Vec3> = (0..tris)
        .map(|f| cross(v, &idx[f * 3..f * 3 + 3]).normalize_or_zero())
        .collect();
    // Each undirected edge: its faces and the corner it starts at.
    let mut edges: BTreeMap<(u32, u32), Vec<(usize, usize)>> = BTreeMap::new();
    for f in 0..tris {
        for k in 0..3 {
            let (a, b) = (id(f, k), id(f, k + 1));
            if a != b {
                edges.entry((a.min(b), a.max(b))).or_default().push((f, k));
            }
        }
    }
    let cos = hard_angle.to_radians().cos();
    let hard_key = |a: u32, b: u32| -> bool {
        edges.get(&(a.min(b), a.max(b))).is_some_and(|e| {
            e.len() == 2
                && normal[e[0].0] != Vec3::ZERO
                && normal[e[1].0] != Vec3::ZERO
                && normal[e[0].0].dot(normal[e[1].0]) < cos
        })
    };
    let hard = |f: usize, k: usize| hard_key(id(f, k), id(f, k + 1));
    let hard_edges: Vec<(u32, u32)> = edges
        .keys()
        .copied()
        .filter(|&(a, b)| hard_key(a, b))
        .collect();
    if hard_edges.is_empty() {
        return None;
    }
    let p = |i: u32| pos(&v[i as usize]);
    let shortest = hard_edges
        .iter()
        .map(|&(a, b)| p(a).distance(p(b)))
        .fold(f32::MAX, f32::min);
    let r = radius.min(shortest * 0.25);
    // Each corner's inset point: away from its face's hard edges by r.
    let mut inset = vec![Vec3::ZERO; tris * 3];
    for f in 0..tris {
        for k in 0..3 {
            let c = |j: usize| pos(&v[idx[f * 3 + j % 3] as usize]);
            let (at, next, prev) = (c(k), c(k + 1), c(k + 2));
            let (un, up) = (next - at, prev - at);
            let (ln, lp) = (un.length(), up.length());
            let mut q = at;
            if ln > 0.0 && lp > 0.0 {
                let (un, up) = (un / ln, up / lp);
                let sin = un.cross(up).length();
                if sin > 1e-4 {
                    let s = r / sin;
                    if hard(f, k) {
                        q += up * s.min(lp * 0.45);
                    }
                    if hard(f, k + 2) {
                        q += un * s.min(ln * 0.45);
                    }
                }
            }
            inset[f * 3 + k] = q;
        }
    }
    // Faces meeting at a position across soft edges share one point (the
    // mean of their insets), so no crack opens between them.
    let mut at_pos: BTreeMap<u32, Vec<(usize, usize)>> = BTreeMap::new();
    for f in 0..tris {
        for k in 0..3 {
            at_pos.entry(id(f, k)).or_default().push((f, k));
        }
    }
    let mut group = vec![usize::MAX; tris * 3];
    let mut groups: Vec<(u32, Vec<usize>)> = Vec::new();
    for (&vid, corners) in &at_pos {
        let mut parent: Vec<usize> = (0..corners.len()).collect();
        fn root(p: &mut [usize], i: usize) -> usize {
            let mut i = i;
            while p[i] != i {
                p[i] = p[p[i]];
                i = p[i];
            }
            i
        }
        for (x, &(fx, kx)) in corners.iter().enumerate() {
            for (y, &(fy, ky)) in corners.iter().enumerate().skip(x + 1) {
                // Two faces at vid share a soft edge out of vid.
                let others = |f: usize, k: usize| [id(f, k + 1), id(f, k + 2)];
                let shared = others(fx, kx)
                    .into_iter()
                    .find(|o| others(fy, ky).contains(o));
                if let Some(o) = shared
                    && !hard_key(vid, o)
                {
                    let (rx, ry) = (root(&mut parent, x), root(&mut parent, y));
                    parent[rx.max(ry)] = rx.min(ry);
                }
            }
        }
        let mut seen: BTreeMap<usize, usize> = BTreeMap::new();
        for (x, &(f, k)) in corners.iter().enumerate() {
            let rt = root(&mut parent, x);
            let g = *seen.entry(rt).or_insert_with(|| {
                groups.push((vid, Vec::new()));
                groups.len() - 1
            });
            group[f * 3 + k] = g;
            groups[g].1.push(f * 3 + k);
        }
    }
    let point: Vec<Vec3> = groups
        .iter()
        .map(|(_, cs)| cs.iter().map(|&c| inset[c]).sum::<Vec3>() / cs.len() as f32)
        .collect();
    // The inset faces, every corner its own vertex.
    let mut nv: Vec<FVertex> = Vec::with_capacity(tris * 3 + hard_edges.len() * 4);
    let mut ns: Vec<u32> = Vec::new();
    let mut ni: Vec<u32> = Vec::new();
    let corner = |c: usize| -> (FVertex, u32) {
        let mut x = v[idx[c] as usize];
        x.p = point[group[c]].to_array();
        (x, src[idx[c] as usize])
    };
    for c in 0..tris * 3 {
        let (x, s) = corner(c);
        ni.push(nv.len() as u32);
        nv.push(x);
        ns.push(s);
    }
    let before = ni.len() / 3;
    let push_tri = |nv: &mut Vec<FVertex>,
                    ns: &mut Vec<u32>,
                    ni: &mut Vec<u32>,
                    t: [(FVertex, u32); 3],
                    up: Vec3| {
        let [a, b, c] = t.map(|x| Vec3::from(x.0.p));
        let n = (b - a).cross(c - a);
        if n.length_squared() < 1e-14 {
            return;
        }
        let order = if n.dot(up) < 0.0 {
            [0, 2, 1]
        } else {
            [0, 1, 2]
        };
        for o in order {
            ni.push(nv.len() as u32);
            nv.push(t[o].0);
            ns.push(t[o].1);
        }
    };
    // A quad across each hard edge.
    for &(a, b) in &hard_edges {
        let e = &edges[&(a, b)];
        let ((f1, k1), (f2, k2)) = (e[0], e[1]);
        let at = |f: usize, k: usize, of: u32| {
            if id(f, k) == of {
                f * 3 + k
            } else {
                f * 3 + (k + 1) % 3
            }
        };
        let (a1, b1, a2, b2) = (at(f1, k1, a), at(f1, k1, b), at(f2, k2, a), at(f2, k2, b));
        let up = normal[f1] + normal[f2];
        push_tri(
            &mut nv,
            &mut ns,
            &mut ni,
            [corner(a1), corner(b1), corner(b2)],
            up,
        );
        push_tri(
            &mut nv,
            &mut ns,
            &mut ni,
            [corner(a1), corner(b2), corner(a2)],
            up,
        );
    }
    // A fan closing each corner where three or more groups meet.
    for (&vid, corners) in &at_pos {
        let gs: Vec<usize> = {
            let mut g: Vec<usize> = corners.iter().map(|&(f, k)| group[f * 3 + k]).collect();
            g.sort_unstable();
            g.dedup();
            g
        };
        if gs.len() < 3 {
            continue;
        }
        let up = corners
            .iter()
            .map(|&(f, _)| normal[f])
            .sum::<Vec3>()
            .normalize_or_zero();
        if up == Vec3::ZERO {
            continue;
        }
        let centre = p(vid);
        let x = up.any_orthonormal_vector();
        let y = up.cross(x);
        let mut ring: Vec<(f32, usize)> = gs
            .iter()
            .map(|&g| {
                let d = point[g] - centre;
                (d.dot(y).atan2(d.dot(x)), g)
            })
            .collect();
        ring.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let first = |g: usize| corner(groups[g].1[0]);
        for w in 1..ring.len() - 1 {
            push_tri(
                &mut nv,
                &mut ns,
                &mut ni,
                [first(ring[0].1), first(ring[w].1), first(ring[w + 1].1)],
                up,
            );
        }
    }
    let added = ni.len() / 3 - before;
    *v = nv;
    *src = ns;
    *idx = ni;
    Some((hard_edges.len(), added))
}

/// Drops the triangles (of unwelded corners) thinner than one step of the
/// grid the cook stores positions on (65,535 steps over the primitive's
/// widest extent): such a face has no normal to keep. It draws nothing,
/// and quantising its corners turns it any way (a lathe's pole triangle,
/// two corners at the pole, decodes as a sliver whose face is not the
/// normal the cook gave it).
fn drop_degenerate(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>) {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for x in v.iter() {
        lo = lo.min(pos(x));
        hi = hi.max(pos(x));
    }
    let step = (hi - lo).max_element() / 65535.0;
    let keep: Vec<bool> = v
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| {
            let [a, b, c] = t.each_ref().map(pos);
            let longest = a.distance(b).max(b.distance(c)).max(c.distance(a));
            (b - a).cross(c - a).length() > step * longest
        })
        .collect();
    if keep.iter().all(|&k| k) {
        return;
    }
    let mut k = keep.iter().flat_map(|&k| [k; 3]);
    v.retain(|_| k.next().unwrap_or(true));
    let mut k = keep.iter().flat_map(|&k| [k; 3]);
    src.retain(|_| k.next().unwrap_or(true));
    *idx = (0..v.len() as u32).collect();
}

/// Sets every corner's normal as the contract shades: corners unwelded
/// (faces too thin to have a normal dropped), the cook welds what came out
/// the same.
fn shade(v: &mut Vec<FVertex>, idx: &mut Vec<u32>, src: &mut Vec<u32>, normals: Normals) {
    unweld(v, idx, src);
    drop_degenerate(v, idx, src);
    let tris = v.len() / 3;
    let face: Vec<Vec3> = (0..tris)
        .map(|f| cross(v, &idx[f * 3..f * 3 + 3]))
        .collect();
    let set = |v: &mut [FVertex], i: usize, n: Vec3| {
        v[i].n = n.normalize_or(Vec3::Y).to_array();
    };
    match normals {
        Normals::Any => {}
        Normals::Flat => {
            for i in 0..v.len() {
                set(v, i, face[i / 3]);
            }
        }
        Normals::Smooth => {
            let mut sum: BTreeMap<([u32; 3], [u32; 2]), Vec3> = BTreeMap::new();
            for i in 0..v.len() {
                *sum.entry((pkey(v[i].p), v[i].uv.map(f32::to_bits)))
                    .or_insert(Vec3::ZERO) += face[i / 3];
            }
            for i in 0..v.len() {
                let n = sum[&(pkey(v[i].p), v[i].uv.map(f32::to_bits))];
                set(v, i, n);
            }
        }
        Normals::Angle(deg) => {
            let cos = deg.to_radians().cos();
            let mut faces_at: BTreeMap<[u32; 3], Vec<usize>> = BTreeMap::new();
            for (i, x) in v.iter().enumerate() {
                faces_at.entry(pkey(x.p)).or_default().push(i / 3);
            }
            for i in 0..v.len() {
                let own = face[i / 3].normalize_or_zero();
                let n: Vec3 = faces_at[&pkey(v[i].p)]
                    .iter()
                    .map(|&f| face[f])
                    .filter(|n| n.normalize_or_zero().dot(own) >= cos - 1e-6)
                    .sum();
                set(v, i, n);
            }
        }
    }
}

/// The primitive as voxels of `edge` (its own units, the grid aligned to
/// the origin); `None` when it has no surface.
fn voxelise(
    v: &[FVertex],
    idx: &[u32],
    surface: &Surface,
    edge: f32,
    palette: &[[u8; 3]],
    max_colors: u32,
) -> Option<(Vec<FVertex>, Vec<u32>, usize)> {
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for &i in idx {
        lo = lo.min(pos(&v[i as usize]));
        hi = hi.max(pos(&v[i as usize]));
    }
    if lo.x > hi.x {
        return None;
    }
    let cell = |p: Vec3| (p / edge).floor().as_ivec3();
    let (c0, c1) = (cell(lo), cell(hi));
    let dims = (c1 - c0 + 1).max(glam::IVec3::ONE);
    if dims.max_element() > 256 {
        return None;
    }
    let (sx, sy, sz) = (dims.x as usize, dims.y as usize, dims.z as usize);
    let at = |c: glam::IVec3| {
        let d = c - c0;
        d.x as usize + sx * (d.y as usize + sy * d.z as usize)
    };
    // The surface: points over every triangle a third of an edge apart.
    let mut sum: Vec<[f32; 5]> = vec![[0.0; 5]; sx * sy * sz];
    for t in idx.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|k| &v[t[k] as usize]);
        let longest = pos(a)
            .distance(pos(b))
            .max(pos(b).distance(pos(c)))
            .max(pos(c).distance(pos(a)));
        let n = ((longest / (edge / 3.0)).ceil() as usize).clamp(1, 4096);
        for i in 0..=n {
            for j in 0..=n - i {
                let (wa, wb) = (i as f32 / n as f32, j as f32 / n as f32);
                let wc = 1.0 - wa - wb;
                let p = pos(a) * wa + pos(b) * wb + pos(c) * wc;
                let uv = [0, 1].map(|k| a.uv[k] * wa + b.uv[k] * wb + c.uv[k] * wc);
                let col = [0, 1, 2, 3].map(|k| a.c[k] * wa + b.c[k] * wb + c.c[k] * wc);
                let s = surface.at(uv, col);
                let e = &mut sum[at(cell(p).clamp(c0, c1))];
                for k in 0..4 {
                    e[k] += s[k];
                }
                e[4] += 1.0;
            }
        }
    }
    // The outside: flood-filled from a border one cell wide round the grid.
    let (px, py, pz) = (sx + 2, sy + 2, sz + 2);
    let pidx = |x: usize, y: usize, z: usize| x + px * (y + py * z);
    let mut outside = vec![false; px * py * pz];
    let solid_surface = |x: usize, y: usize, z: usize| {
        (1..=sx).contains(&x)
            && (1..=sy).contains(&y)
            && (1..=sz).contains(&z)
            && sum[(x - 1) + sx * ((y - 1) + sy * (z - 1))][4] > 0.0
    };
    let mut stack = vec![(0usize, 0usize, 0usize)];
    outside[0] = true;
    while let Some((x, y, z)) = stack.pop() {
        let mut go = |x: usize, y: usize, z: usize| {
            if !outside[pidx(x, y, z)] && !solid_surface(x, y, z) {
                outside[pidx(x, y, z)] = true;
                stack.push((x, y, z));
            }
        };
        if x > 0 {
            go(x - 1, y, z);
        }
        if x + 1 < px {
            go(x + 1, y, z);
        }
        if y > 0 {
            go(x, y - 1, z);
        }
        if y + 1 < py {
            go(x, y + 1, z);
        }
        if z > 0 {
            go(x, y, z - 1);
        }
        if z + 1 < pz {
            go(x, y, z + 1);
        }
    }
    // The surface cells' colours, held to the palette or cut, then indexed.
    let surface_cells: Vec<usize> = (0..sum.len()).filter(|&i| sum[i][4] > 0.0).collect();
    let mut colors: Vec<[f32; 4]> = surface_cells
        .iter()
        .map(|&i| {
            let s = sum[i];
            [s[0] / s[4], s[1] / s[4], s[2] / s[4], 1.0]
        })
        .collect();
    let max = if max_colors > 0 {
        max_colors.min(255)
    } else {
        255
    };
    reduce(&mut colors, palette, max);
    if !palette.is_empty() && palette.len() > 255 {
        reduce(&mut colors, &[], 255);
    }
    let key = |c: &[f32; 4]| [to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2])];
    let mut index: BTreeMap<[u8; 3], u8> = BTreeMap::new();
    for c in &colors {
        let n = index.len() as u8 + 1;
        index.entry(key(c)).or_insert(n);
    }
    let mut table = vec![[0f32; 3]; 256];
    for (k, &i) in &index {
        table[i as usize] = k.map(to_linear);
    }
    let mut grid = vec![0u8; sum.len()];
    for (n, &i) in surface_cells.iter().enumerate() {
        grid[i] = index[&key(&colors[n])];
    }
    let mut model = VoxModel {
        size: [sx as u32, sy as u32, sz as u32],
        voxels: Vec::new(),
    };
    for z in 0..sz {
        for y in 0..sy {
            for x in 0..sx {
                let i = x + sx * (y + sy * z);
                let inside = !outside[pidx(x + 1, y + 1, z + 1)];
                let c = if grid[i] != 0 {
                    grid[i]
                } else if inside {
                    1
                } else {
                    continue;
                };
                model.voxels.push([x as u8, y as u8, z as u8, c]);
            }
        }
    }
    let count = model.voxels.len();
    let mut out_v = Vec::new();
    let mut out_i = Vec::new();
    for q in mesh_model(&model) {
        let base = out_v.len() as u32;
        let col = table[q.color as usize];
        for c in q.corners {
            out_v.push(FVertex {
                p: [0, 1, 2].map(|a| (c0[a] as f32 + c[a]) * edge),
                n: q.normal,
                uv: [0.0; 2],
                c: [col[0], col[1], col[2], 1.0],
                t: [0.0; 4],
                key: [0, 0],
            });
        }
        out_i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Some((out_v, out_i, count))
}

/// The model-space transform of the first node of the default scene that
/// draws `prim`'s mesh (identity when none does): voxels sit on one grid
/// across the model's parts, not one per part.
fn placement_of(src: &GltfSource, prim: &gltf::Primitive) -> glam::Mat4 {
    use glam::{Mat4, Quat};
    let key = |p: &gltf::Primitive| {
        (
            p.get(&gltf::Semantic::Positions).map(|a| a.index()),
            p.indices().map(|a| a.index()),
        )
    };
    let Some(mesh) = src
        .doc
        .meshes()
        .find(|m| m.primitives().any(|p| key(&p) == key(prim)))
    else {
        return Mat4::IDENTITY;
    };
    let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) else {
        return Mat4::IDENTITY;
    };
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    stack.reverse();
    while let Some((node, parent)) = stack.pop() {
        let (t, r, s) = node.transform().decomposed();
        let world = parent
            * Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
        if node.mesh().is_some_and(|m| m.index() == mesh.index()) {
            // A skinned mesh is placed by its bind pose (model.rs `skin_binds`).
            return crate::cook::model::skin_binds(src)
                .get(&node.index())
                .copied()
                .unwrap_or(world);
        }
        let mut children: Vec<_> = node.children().map(|c| (c, world)).collect();
        children.reverse();
        stack.extend(children);
    }
    Mat4::IDENTITY
}

/// The triangles a model draws (each primitive times the default scene's
/// nodes that draw its mesh, at least once): the face budget's measure.
pub(in crate::cook) fn drawn_triangles(src: &GltfSource) -> usize {
    let mut uses = vec![0usize; src.doc.meshes().count()];
    if let Some(scene) = src.doc.default_scene().or_else(|| src.doc.scenes().next()) {
        let mut stack: Vec<gltf::Node> = scene.nodes().collect();
        while let Some(n) = stack.pop() {
            if let Some(m) = n.mesh() {
                uses[m.index()] += 1;
            }
            stack.extend(n.children());
        }
    }
    src.doc
        .meshes()
        .map(|m| {
            uses[m.index()].max(1)
                * m.primitives()
                    .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
                    .map(|p| triangles_of(p))
                    .sum::<usize>()
        })
        .sum()
}

fn triangles_of(p: gltf::Primitive) -> usize {
    p.indices()
        .map(|a| a.count())
        .or_else(|| p.get(&gltf::Semantic::Positions).map(|a| a.count()))
        .unwrap_or(0)
        / 3
}

/// Conforms one primitive's vertices (`v`, before welding), triangles and
/// source vertices to the contract. The cook keeps normals and colours
/// the answer says were set.
pub(in crate::cook) fn primitive(
    c: &Conform,
    src: &GltfSource,
    prim: &gltf::Primitive,
    v: &mut Vec<FVertex>,
    idx: &mut Vec<u32>,
    sources: &mut Vec<u32>,
) -> Changes {
    let mut ch = Changes {
        faces: (idx.len() / 3, idx.len() / 3),
        ..Changes::default()
    };
    let skinned = v.iter().any(|x| x.key != [0, 0]);
    let surface = Surface::of(src, prim);
    let (budget, _) = c.budget(src);
    let plan = Plan::get(c, src);
    let target = (budget > 0).then(|| {
        plan.targets
            .get(&prim_key(prim))
            .copied()
            .unwrap_or(usize::MAX)
    });
    let error = local_error(src, prim, plan.error.0);
    let textured = textured(prim);
    let mut reshade = false;
    if c.voxel > 0.0 && skinned {
        ch.skipped
            .push("not voxelised: it is skinned or morphed".into());
    }
    if c.voxel > 0.0 && !skinned {
        // Voxelised in model space, then brought back under the node:
        // positions by the placement's inverse, normals by that inverse's
        // inverse transpose, the placement's transpose.
        let place = placement_of(src, prim);
        let inverse = place.inverse();
        let normal_back = glam::Mat3::from_mat4(place).transpose();
        let placed: Vec<FVertex> = v
            .iter()
            .map(|x| FVertex {
                p: place.transform_point3(pos(x)).to_array(),
                ..*x
            })
            .collect();
        // The plan is made from the source's triangles, so a model within its
        // budget before voxelising has no target: its voxels still answer to
        // the budget, this primitive's share of it by source triangles.
        let voxel_target = target.map(|t| {
            if t != usize::MAX {
                t
            } else {
                (budget as usize * (idx.len() / 3))
                    .div_ceil(drawn_triangles(src).max(1))
                    .max(1)
            }
        });
        let mut edge = c.voxel;
        for _ in 0..5 {
            match voxelise(&placed, idx, &surface, edge, &c.palette, c.max_colors) {
                Some((nv, ni, count))
                    if voxel_target.is_none_or(|t| ni.len() / 3 <= t) || edge >= c.voxel * 16.0 =>
                {
                    ch.distinct = nv
                        .iter()
                        .map(|x| x.c.map(f32::to_bits))
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    *sources = vec![0; nv.len()];
                    *v = nv
                        .into_iter()
                        .map(|x| FVertex {
                            p: inverse.transform_point3(pos(&x)).to_array(),
                            n: (normal_back * Vec3::from(x.n))
                                .normalize_or(Vec3::Y)
                                .to_array(),
                            ..x
                        })
                        .collect();
                    *idx = ni;
                    ch.voxel = Some((edge, count));
                    ch.normals = true;
                    ch.colors = true;
                    break;
                }
                // Past the budget or the 256-cell grid: coarser.
                _ => edge *= 2.0,
            }
        }
        if ch.voxel.is_some() {
            ch.faces.1 = idx.len() / 3;
            return ch;
        }
        ch.skipped
            .push("not voxelised: wider than 256 voxels of 16 times the edge".into());
    }
    if let Some(t) = target.filter(|&t| t != usize::MAX) {
        reshade |= decimate(v, idx, sources, t, error, textured);
    }
    if c.bevel > 0.0 {
        let hard = match c.normals {
            Normals::Angle(a) => a,
            _ => 30.0,
        };
        let before = (v.clone(), idx.clone(), sources.clone());
        if let Some(b) = bevel(v, idx, sources, c.bevel, hard) {
            ch.bevel = Some(b);
            // The chamfers count against the budget: simplify the mesh
            // under them and bevel again.
            if let Some(t) = target.filter(|&t| t != usize::MAX)
                && idx.len() / 3 > t
            {
                (*v, *idx, *sources) = before;
                reshade |= decimate(
                    v,
                    idx,
                    sources,
                    t.saturating_sub(b.1).max(t / 2),
                    error,
                    textured,
                );
                ch.bevel = bevel(v, idx, sources, c.bevel, hard);
                if idx.len() / 3 > t {
                    reshade |= decimate(v, idx, sources, t, error, textured);
                    ch.skipped
                        .push("the chamfers were simplified to stay within the budget".into());
                }
            }
            if c.normals == Normals::Any {
                // The chamfers' own faces need normals: each its face's.
                shade(v, idx, sources, Normals::Angle(hard));
                ch.normals = true;
            }
        }
    }
    if c.bakes() {
        let flat = c.normals == Normals::Flat;
        if flat {
            unweld(v, idx, sources);
        }
        let mut colors: Vec<[f32; 4]> = if flat {
            v.as_chunks::<3>()
                .0
                .iter()
                .flat_map(|t| {
                    let m = surface.over([&t[0], &t[1], &t[2]]);
                    [m; 3]
                })
                .collect()
        } else {
            v.iter().map(|x| surface.at(x.uv, x.c)).collect()
        };
        ch.distinct = reduce(&mut colors, &c.palette, c.max_colors);
        for (x, col) in v.iter_mut().zip(colors) {
            x.c = col;
        }
        ch.colors = true;
    }
    if c.normals != Normals::Any {
        shade(v, idx, sources, c.normals);
        ch.normals = true;
    } else if reshade && !ch.normals {
        // Simplified welded by position: the corners' normals are their
        // position's first vertex's, so each face's are made again.
        shade(v, idx, sources, Normals::Angle(30.0));
        ch.normals = true;
    }
    ch.faces.1 = idx.len() / 3;
    ch
}

/// The report's lines for a model's conformed primitives.
pub(in crate::cook) fn notes(c: &Conform, src: &GltfSource, changes: &[Changes]) -> Vec<String> {
    if changes.is_empty() {
        return Vec::new();
    }
    let (before, after): (usize, usize) = changes
        .iter()
        .fold((0, 0), |(a, b), x| (a + x.faces.0, b + x.faces.1));
    let changed = c.line.is_some()
        || src.segments.is_some()
        || changes.iter().any(|x| {
            x.faces.0 != x.faces.1
                || x.normals
                || x.colors
                || x.voxel.is_some()
                || x.bevel.is_some()
        });
    if !changed {
        return vec![format!("conform: {UNCHANGED}")];
    }
    let mut what = vec![format!("{before} → {after} triangles")];
    if let Some((detail, written)) = src.segments {
        what.push(format!(
            "built at {:.0} % of its segments for the budget ({written} drawn as written)",
            detail * 100.0
        ));
    }
    let (budget, name) = c.budget(src);
    let mut warning = None;
    if budget > 0 {
        let drawn = drawn_triangles(src);
        let plan = Plan::get(c, src);
        what.push(format!(
            "ArtShape.{name} {budget} ({drawn} drawn before{})",
            match plan.over {
                Some(_) => ", over budget",
                None if drawn > budget as usize => ", decimated",
                None => ", within",
            }
        ));
        if let Some(floor) = plan.over {
            warning = Some(format!(
                "conform: warning: over budget: ArtShape.{name} {budget} is not met without losing the shape: simplified as far as {} ({:.3} m) lets it, {floor} triangles drawn (ArtShape.silhouette sets the bound; a lighter source meets it)",
                plan.error.1, plan.error.0
            ));
        }
    }
    if c.normals != Normals::Any {
        what.push(format!("normals {:?}", c.normals).to_lowercase());
    }
    let voxels: usize = changes.iter().filter_map(|x| x.voxel).map(|x| x.1).sum();
    if let Some(edge) = changes
        .iter()
        .filter_map(|x| x.voxel)
        .map(|x| x.0)
        .reduce(f32::max)
    {
        what.push(format!(
            "voxelised at {edge} m ({voxels} voxels{})",
            if edge > c.voxel {
                format!(", ArtShape.voxel {} coarsened to fit", c.voxel)
            } else {
                String::new()
            }
        ));
    }
    let (edges, added) = changes
        .iter()
        .filter_map(|x| x.bevel)
        .fold((0, 0), |(a, b), x| (a + x.0, b + x.1));
    if edges > 0 {
        what.push(format!(
            "bevel {} m on {edges} hard edges (+{added} triangles)",
            c.bevel
        ));
    }
    if c.bakes() {
        what.push(format!(
            "vertex colours baked, {} distinct ({}), materials drawn white without a base colour texture",
            changes.iter().map(|x| x.distinct).max().unwrap_or(0),
            if c.palette.is_empty() {
                format!("ArtPalette.maxColors {}", c.max_colors)
            } else {
                format!("on ArtPalette.colors' {}", c.palette.len())
            }
        ));
    }
    if let Some(l) = &c.line {
        what.push(format!("outline hull data for ArtLine width {}", l.width));
    }
    let mut out = vec![format!("conform: {}", what.join("; "))];
    out.extend(warning);
    let mut skipped: Vec<&String> = changes.iter().flat_map(|x| &x.skipped).collect();
    skipped.sort();
    skipped.dedup();
    out.extend(skipped.into_iter().map(|s| format!("conform: {s}")));
    out
}

/// Per cooked vertex, the area-weighted normal over every corner at its
/// position: the direction the outline hull extrudes along.
pub(in crate::cook) fn hull_normals(pos: &[[f32; 3]], idx: &[u32]) -> Vec<[f32; 3]> {
    let mut sum: BTreeMap<[u32; 3], Vec3> = BTreeMap::new();
    for t in idx.as_chunks::<3>().0 {
        let [a, b, c] = [0, 1, 2].map(|k| Vec3::from(pos[t[k] as usize]));
        let n = (b - a).cross(c - a);
        for &i in t {
            *sum.entry(pkey(pos[i as usize])).or_insert(Vec3::ZERO) += n;
        }
    }
    pos.iter()
        .map(|p| {
            sum.get(&pkey(*p))
                .copied()
                .unwrap_or(Vec3::Y)
                .normalize_or(Vec3::Y)
                .to_array()
        })
        .collect()
}

/// The `outline` section's header (the module docs give the layout).
pub(in crate::cook) fn hull_header(l: &Line, primitives: usize) -> Vec<u8> {
    let words: [u32; 8] = [
        1,
        l.width.to_bits(),
        l.color[0].to_bits(),
        l.color[1].to_bits(),
        l.color[2].to_bits(),
        l.place,
        l.distance.to_bits(),
        primitives as u32,
    ];
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}
