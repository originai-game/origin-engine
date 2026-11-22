//! A solid rebuilt from a subset or a remap of its triangles and vertices, its attributes and skin carried.

use super::*;

/// The solid without the vertices no triangle uses (canonical order kept
/// once canonicalized again).
pub(crate) fn drop_unused(s: &Solid) -> Solid {
    let mut remap = vec![u32::MAX; s.positions.len()];
    let mut positions = Vec::new();
    for t in &s.triangles {
        for &v in t {
            if remap[v as usize] == u32::MAX {
                remap[v as usize] = positions.len() as u32;
                positions.push(s.positions[v as usize]);
            }
        }
    }
    let all: Vec<usize> = (0..s.triangles.len()).collect();
    rebuild(s, &all, &remap, positions)
}

/// The solid with only the triangles `keep` (indices, in order) and the
/// vertices they use: one copy, not a rebuilt one and then a compacted one.
pub(crate) fn kept(s: &Solid, keep: &[usize]) -> Solid {
    let mut remap = vec![u32::MAX; s.positions.len()];
    let mut positions = Vec::new();
    for &i in keep {
        for &v in &s.triangles[i] {
            if remap[v as usize] == u32::MAX {
                remap[v as usize] = positions.len() as u32;
                positions.push(s.positions[v as usize]);
            }
        }
    }
    rebuild(s, keep, &remap, positions)
}

pub(crate) fn rebuild(
    s: &Solid,
    triangles: &[usize],
    remap: &[u32],
    positions: Vec<DVec3>,
) -> Solid {
    let mut out = Solid {
        positions,
        uvs: s.uvs.as_ref().map(|_| Vec::new()),
        colors: s.colors.as_ref().map(|_| Vec::new()),
        ..Default::default()
    };
    for &i in triangles {
        let tags: Vec<&str> = s.face_tags[i].iter().map(|&x| s.names.name(x)).collect();
        let t = s.triangles[i].map(|v| remap[v as usize]);
        match &s.uvs {
            Some(u) => out.push_triangle_uv(t, s.surfaces[i], &tags, u[i]),
            None => out.push_triangle(t, s.surfaces[i], &tags),
        }
        if let Some(c) = &s.colors {
            out.set_colors(out.triangles.len() - 1, c[i]);
        }
    }
    for (&(a, b), tags) in &s.edge_tags {
        let (a, b) = (remap[a as usize], remap[b as usize]);
        if a != b && a != u32::MAX && b != u32::MAX {
            let tags: Vec<&str> = tags.iter().map(|&x| s.names.name(x)).collect();
            out.tag_edge(a, b, &tags);
        }
    }
    out.skin = s
        .skin
        .as_ref()
        .map(|k| remap_skin(k, remap, out.positions.len()));
    out.morphs = s
        .morphs
        .as_ref()
        .map(|m| m.remapped(remap, out.positions.len()));
    out
}

/// A skin's weights moved to new vertex indices (`remap[old]`, or
/// `u32::MAX` for gone); vertices merged into one take the mean of their
/// weights (`Skin::weld`).
pub(crate) fn remap_skin(k: &Skin, remap: &[u32], vertices: usize) -> Skin {
    let mut copies: Vec<Vec<Weights>> = vec![Vec::new(); vertices];
    for (old, &new) in remap.iter().enumerate() {
        if let (Some(c), Some(w)) = (copies.get_mut(new as usize), k.weights.get(old)) {
            c.push(*w);
        }
    }
    Skin {
        bones: k.bones.clone(),
        weights: copies.into_iter().map(|mut c| Skin::weld(&mut c)).collect(),
    }
}
