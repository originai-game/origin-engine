//! dent: a solid pressed in by a tool's shape.

use super::*;

/// Round hollows pressed into the surface.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dent {
    /// Centres, on the surface.
    pub at: Vec<DVec3>,
    /// Radius of each hollow.
    pub radius: f64,
    /// Depth at the centre.
    pub depth: f64,
}

/// Presses the surface in at each point: vertices within `radius` move
/// against the local normal by `depth · (1 − (r/radius)²)²`. Triangles in
/// reach are first split (shared edge midpoints, so no cracks) until no
/// edge is longer than `radius / 4`. Moved faces get `dent`.
pub fn dent(s: &Solid, d: &Dent) -> Result<Solid, EditError> {
    dent_built(s, d)
}

/// [`dent`], with what the result alone does not say ([`EditReport`]: the
/// result's own crossing pairs, a sweep over its triangles).
pub fn dent_report(s: &Solid, d: &Dent) -> Result<(Solid, EditReport), EditError> {
    let out = dent_built(s, d)?;
    let report = EditReport {
        crossing_pairs: crossing_pairs(&out),
    };
    Ok((out, report))
}

pub(crate) fn dent_built(s: &Solid, d: &Dent) -> Result<Solid, EditError> {
    if crate::mesh::rides(s) {
        let mut out = dent_built(&crate::mesh::bare(s), d)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok(out);
    }
    if !positive(d.radius) {
        return Err(EditError::NotPositive("radius"));
    }
    if !positive(d.depth) {
        return Err(EditError::NotPositive("depth"));
    }
    let mut out = s.clone();
    let max_edge = d.radius / 4.0;
    let near = |p: DVec3| d.at.iter().any(|c| (p - *c).length() < d.radius + max_edge);
    for _ in 0..12 {
        let mut split_at: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        for t in &out.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                let (pa, pb) = (out.positions[a as usize], out.positions[b as usize]);
                if (pa - pb).length() > max_edge && (near(pa) || near(pb) || near((pa + pb) / 2.0))
                {
                    split_at.insert(edge_key(a, b), 0);
                }
            }
        }
        if split_at.is_empty() {
            break;
        }
        for (&(a, b), mid) in split_at.iter_mut() {
            *mid = out.push_vertex((out.positions[a as usize] + out.positions[b as usize]) / 2.0);
        }
        let mut next = Solid {
            positions: out.positions.clone(),
            edge_tags: BTreeMap::new(),
            names: Default::default(),
            ..Default::default()
        };
        for (i, t) in out.triangles.iter().enumerate() {
            let tags: Vec<&str> = out.face_tags[i]
                .iter()
                .map(|&x| out.names.name(x))
                .collect();
            let m = [0, 1, 2].map(|k| split_at.get(&edge_key(t[k], t[(k + 1) % 3])).copied());
            for tri in split_triangle(*t, m) {
                match &out.uvs {
                    // Each piece's corners are its triangle's corners or
                    // midpoints of its edges: their UVs, or the means.
                    Some(uvs) => {
                        let uv = piece_uvs(*t, &uvs[i], tri, &split_at);
                        next.push_triangle_uv(tri, out.surfaces[i], &tags, uv);
                    }
                    None => next.push_triangle(tri, out.surfaces[i], &tags),
                }
            }
        }
        // Named edges: each half keeps the name.
        for (&(a, b), tags) in &out.edge_tags {
            let names: Vec<&str> = tags.iter().map(|&x| out.names.name(x)).collect();
            match split_at.get(&(a, b)) {
                Some(&m) => {
                    next.tag_edge(a, m, &names);
                    next.tag_edge(m, b, &names);
                }
                None => next.tag_edge(a, b, &names),
            }
        }
        out = next;
    }
    // Vertex normals (area-weighted) for the push direction.
    let mut normals = vec![DVec3::ZERO; out.positions.len()];
    for t in &out.triangles {
        let [a, b, c] = t.map(|v| out.positions[v as usize]);
        let n = (b - a).cross(c - a);
        for &v in t {
            normals[v as usize] += n;
        }
    }
    let mut moved = vec![false; out.positions.len()];
    for (v, p) in out.positions.clone().iter().enumerate() {
        let mut push = 0.0;
        for c in &d.at {
            let r = (*p - *c).length() / d.radius;
            if r < 1.0 {
                let f = 1.0 - r * r;
                push += d.depth * f * f;
            }
        }
        if push > 0.0 {
            out.positions[v] = *p - normals[v].normalize_or_zero() * push;
            moved[v] = true;
        }
    }
    let dent_tag = out.names.intern("dent");
    for (i, t) in out.triangles.iter().enumerate() {
        if t.iter().any(|&v| moved[v as usize]) && !out.face_tags[i].contains(&dent_tag) {
            let tags = std::sync::Arc::make_mut(&mut out.face_tags[i]);
            tags.push(dent_tag);
            tags.sort_unstable();
        }
    }
    out.canonicalize()
        .map_err(|_| EditError::NotPositive("depth"))?;
    Ok(out)
}
