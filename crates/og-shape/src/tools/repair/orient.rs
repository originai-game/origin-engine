//! Step: faces turned so neighbours agree and the solid faces out.

use super::*;

/// Turns triangles over so that neighbours across every manifold edge
/// (two triangles) agree, piece by piece from each piece's first
/// triangle, then turns each piece over whole if its signed volume (about
/// its vertices' mean) is negative, so closed pieces face out. Edges with
/// more than two triangles do not propagate. Returns the solid and how
/// many triangles were turned.
pub fn orient(s: &Solid) -> Result<(Solid, usize), RepairError> {
    orient_owned(s.clone())
}

/// [`orient`], on a solid it may change in place.
pub(crate) fn orient_owned(mut s: Solid) -> Result<(Solid, usize), RepairError> {
    s.canonicalize()?;
    let n = s.triangles.len();
    let mut by_edge: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, t) in s.triangles.iter().enumerate() {
        for k in 0..3 {
            by_edge
                .entry(edge_key(t[k], t[(k + 1) % 3]))
                .or_default()
                .push(i as u32);
        }
    }
    // Whether triangle i runs a → b.
    let runs = |t: [u32; 3], a: u32, b: u32| (0..3).any(|k| t[k] == a && t[(k + 1) % 3] == b);
    let mut flip = vec![false; n];
    let mut seen = vec![false; n];
    let mut piece_of = vec![u32::MAX; n];
    let mut pieces: Vec<Vec<u32>> = Vec::new();
    for start in 0..n {
        if seen[start] {
            continue;
        }
        let id = pieces.len() as u32;
        let mut members = vec![start as u32];
        seen[start] = true;
        piece_of[start] = id;
        let mut head = 0;
        while head < members.len() {
            let i = members[head] as usize;
            head += 1;
            let t = s.triangles[i];
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                let users = &by_edge[&edge_key(a, b)];
                if users.len() != 2 {
                    continue;
                }
                let j = if users[0] as usize == i {
                    users[1]
                } else {
                    users[0]
                } as usize;
                if seen[j] {
                    continue;
                }
                // Neighbours agree when they run the edge opposite ways.
                let same_way = runs(s.triangles[j], a, b);
                flip[j] = flip[i] ^ same_way;
                seen[j] = true;
                piece_of[j] = id;
                members.push(j as u32);
            }
        }
        pieces.push(members);
    }
    for members in &pieces {
        let mut verts: Vec<u32> = members
            .iter()
            .flat_map(|&i| s.triangles[i as usize])
            .collect();
        verts.sort_unstable();
        verts.dedup();
        let centre = verts
            .iter()
            .map(|&v| s.positions[v as usize])
            .sum::<DVec3>()
            / verts.len() as f64;
        let mut volume = 0.0;
        for &i in members {
            let t = s.triangles[i as usize];
            let [a, b, c] = t.map(|v| s.positions[v as usize] - centre);
            let v = a.dot(b.cross(c));
            volume += if flip[i as usize] { -v } else { v };
        }
        if volume < 0.0 {
            for &i in members {
                flip[i as usize] = !flip[i as usize];
            }
        }
    }
    let mut turned = 0;
    for (i, &f) in flip.iter().enumerate() {
        if f {
            s.reverse_triangle(i);
            turned += 1;
        }
    }
    s.canonicalize()?;
    Ok((s, turned))
}
