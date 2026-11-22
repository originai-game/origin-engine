//! shell: a solid hollowed to a wall thickness, faces opened by name.

use super::*;

mod collapse;
mod faces;
mod inset;
mod pieces;

/// Hollowing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shell {
    /// Wall thickness.
    pub thickness: f64,
    /// Faces to take away: names (`top`), or a direction (`+x`, `-y`, …)
    /// for the flat face furthest that way. Empty for a closed void.
    pub open: Vec<String>,
}

/// cos 1°: normals closer than this are one plane for the offset.
pub(crate) const SAME_PLANE_COS: f64 = 0.999_847_695_156_391_3;

/// cos 0.5°: how flat a face to open must be.
pub(crate) const FLAT_COS: f64 = 0.999_961_923_064_171_3;

pub(crate) fn tri_normal(s: &Solid, t: [u32; 3]) -> DVec3 {
    let [a, b, c] = t.map(|v| s.positions[v as usize]);
    (b - a).cross(c - a).normalize_or_zero()
}

/// Triangles to open: by name, or the flat face furthest along a direction.
pub(crate) fn opened(s: &Solid, open: &[String]) -> Result<Vec<bool>, EditError> {
    let mut mask = vec![false; s.triangles.len()];
    for term in open {
        let dir = match term.to_ascii_lowercase().as_str() {
            "+x" => Some(DVec3::X),
            "-x" => Some(DVec3::NEG_X),
            "+y" => Some(DVec3::Y),
            "-y" => Some(DVec3::NEG_Y),
            "+z" => Some(DVec3::Z),
            "-z" => Some(DVec3::NEG_Z),
            _ => None,
        };
        let picked: Vec<usize> = match dir {
            Some(dir) => {
                let far = s
                    .positions
                    .iter()
                    .map(|p| p.dot(dir))
                    .fold(f64::NEG_INFINITY, f64::max);
                let tol = size(s) * 1e-9;
                (0..s.triangles.len())
                    .filter(|&i| {
                        let t = s.triangles[i];
                        tri_normal(s, t).dot(dir) > FLAT_COS
                            && t.iter()
                                .all(|&v| (s.positions[v as usize].dot(dir) - far).abs() <= tol)
                    })
                    .collect()
            }
            None => s
                .faces(&[term])
                .map_err(|e| EditError::NoSuchFace(e.missing, e.available))?
                .into_iter()
                .map(|i| i as usize)
                .collect(),
        };
        if picked.is_empty() {
            return Err(EditError::NotFlat(term.clone()));
        }
        let n0 = tri_normal(s, s.triangles[picked[0]]);
        if picked
            .iter()
            .any(|&i| tri_normal(s, s.triangles[i]).dot(n0) < FLAT_COS)
        {
            return Err(EditError::NotFlat(term.clone()));
        }
        for i in picked {
            mask[i] = true;
        }
    }
    Ok(mask)
}

/// The displacement `d` with `n·d = target` for every plane, in the least
/// squares sense over the distinct planes (1, 2 or 3+).
pub(crate) fn offset(planes: &[(DVec3, f64)]) -> DVec3 {
    // Cluster normals that are one plane; a cluster with an open face
    // keeps the open face's target.
    let mut clusters: Vec<(DVec3, f64, bool)> = Vec::new();
    for &(n, target) in planes {
        // A triangle of no area (a sliver a boolean left) has no plane.
        if n == DVec3::ZERO {
            continue;
        }
        let open = target > 0.0;
        match clusters
            .iter_mut()
            .find(|c| c.0.normalize().dot(n) > SAME_PLANE_COS)
        {
            Some(c) => {
                c.0 += n;
                if open {
                    c.1 = target;
                    c.2 = true;
                }
            }
            None => clusters.push((n, target, open)),
        }
    }
    let cs: Vec<(DVec3, f64)> = clusters.iter().map(|c| (c.0.normalize(), c.1)).collect();
    fit(&cs)
}

/// The displacement `d` with `n·d = target` for the distinct unit normals
/// given: along the one, in the plane of the two, the point of the three,
/// or the least squares fit of more.
pub(crate) fn fit(cs: &[(DVec3, f64)]) -> DVec3 {
    match cs.len() {
        0 => DVec3::ZERO,
        1 => cs[0].0 * cs[0].1,
        2 => {
            // d = a n1 + b n2 with n1·d = t1, n2·d = t2.
            let (n1, t1, n2, t2) = (cs[0].0, cs[0].1, cs[1].0, cs[1].1);
            let k = n1.dot(n2);
            let det = 1.0 - k * k;
            if det.abs() < 1e-12 {
                return n1 * t1;
            }
            let a = (t1 - k * t2) / det;
            let b = (t2 - k * t1) / det;
            n1 * a + n2 * b
        }
        _ => {
            // Normal equations (Σ n nᵀ) d = Σ t n, by Cramer's rule.
            let mut m = [[0.0f64; 3]; 3];
            let mut r = [0.0f64; 3];
            for &(n, t) in cs {
                let n = n.to_array();
                for i in 0..3 {
                    r[i] += t * n[i];
                    for j in 0..3 {
                        m[i][j] += n[i] * n[j];
                    }
                }
            }
            let det3 = |m: &[[f64; 3]; 3]| {
                m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                    - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                    + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
            };
            let det = det3(&m);
            if det.abs() < 1e-12 {
                return cs.iter().map(|c| c.0 * c.1).sum::<DVec3>() / cs.len() as f64;
            }
            let col = |k: usize| {
                let mut mk = m;
                for i in 0..3 {
                    mk[i][k] = r[i];
                }
                det3(&mk) / det
            };
            DVec3::new(col(0), col(1), col(2))
        }
    }
}

/// Hollows the solid, leaving walls `thickness` thick. Each vertex of an
/// inner solid moves by the least-squares fit over the planes it touches:
/// `thickness` in from kept faces, `thickness` out through opened ones (so
/// the cut opens them); then the inner solid is cut away. Inner faces get
/// `cut` and `shell.*`.
///
/// Where that inner solid folds through itself (a face narrower than the
/// wall turns over), a clean one is looked for in turn: its outside
/// (`self_union`, `docs/research.md` §27); for a convex solid, the space
/// inside its planes (`inset`, §35); the copy with the turned faces
/// collapsed (`collapse`, §37); the solid taken apart at its holes, each
/// piece hollowed and the pockets cut from the bodies (`pieces`, §38:
/// a hollow that pinches off). If none is clean the result comes back
/// with its crossing pairs counted. A convex solid whose walls would meet
/// is refused ([`EditError::NoRoom`]).
pub fn shell(s: &Solid, d: &Shell) -> Result<Solid, EditError> {
    shell_report(s, d).map(|(s, _)| s)
}

/// [`shell`], with what the result alone does not say ([`EditReport`]).
pub fn shell_report(s: &Solid, d: &Shell) -> Result<(Solid, EditReport), EditError> {
    if crate::mesh::rides(s) {
        let (mut out, report) = shell_report(&crate::mesh::bare(s), d)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok((out, report));
    }
    shell_built(s, d, true)
}

/// [`shell_report`] of a bare solid. `again`: where nothing else gives
/// a clean hollow, the whole is tried once more at a wall a millionth
/// (then a hundred-thousandth) thicker or thinner.
pub(crate) fn shell_built(
    s: &Solid,
    d: &Shell,
    again: bool,
) -> Result<(Solid, EditReport), EditError> {
    let t = d.thickness;
    if !positive(t) {
        return Err(EditError::NotPositive("thickness"));
    }
    let open = opened(s, &d.open)?;
    let mut planes: Vec<Vec<(DVec3, f64)>> = vec![Vec::new(); s.positions.len()];
    for (i, tri) in s.triangles.iter().enumerate() {
        let [a, b, c] = tri.map(|v| s.positions[v as usize]);
        let area_n = (b - a).cross(c - a);
        let target = if open[i] { t } else { -t };
        for &v in tri {
            planes[v as usize].push((area_n.normalize_or_zero(), target));
        }
    }
    // A convex solid's inner solid is known without a copy to fold: the
    // space inside its planes, each moved by the wall (`inset`). With no
    // such space the walls meet, and a copy turned inside out would be
    // cut as if it were a hollow.
    let targets: Vec<f64> = open.iter().map(|&o| if o { t } else { -t }).collect();
    let exact = match inset::inset(s, &targets, &[], &[], size(s)) {
        inset::Inset::NoRoom(short) => {
            return Err(EditError::NoRoom {
                thickness: t,
                short,
            });
        }
        inset::Inset::Inner(i) => Some(*i),
        inset::Inset::Unknown => None,
    };
    let mut inner = Solid::default();
    for (v, p) in s.positions.iter().enumerate() {
        inner.push_vertex(*p + offset(&planes[v]));
    }
    for (i, tri) in s.triangles.iter().enumerate() {
        inner.push_triangle(*tri, s.surfaces[i], &["wall"]);
    }
    // The inner walls take the outer ones' UVs, corner for corner.
    inner.uvs.clone_from(&s.uvs);
    let copied = inner.canonicalize().is_ok();
    let cut = |tool: &Solid| {
        uv_boolean(
            s,
            tool,
            &Boolean {
                op: BoolOp::Cut,
                target: "",
                tool: "shell",
            },
            "the shell",
        )
    };
    if !copied {
        // A vertex with no finite place (a sliver a boolean left has no
        // plane to speak of): no copy. The planes and the pieces pass
        // over slivers.
        let clean = |i: &Solid| {
            cut(i)
                .ok()
                .filter(|(_, r)| r.touching_edges == 0)
                .map(|(out, _)| {
                    let crossing_pairs = crossing_pairs(&out);
                    (out, EditReport { crossing_pairs })
                })
                .filter(|e| e.1.crossing_pairs == 0)
        };
        if let Some(e) = exact.as_ref().and_then(clean) {
            return Ok(e);
        }
        if let Some(e) = pieces::pieced(s, &targets, t, size(s))
            .as_ref()
            .and_then(clean)
        {
            return Ok(e);
        }
        return Err(EditError::Unsupported(
            "a face of the solid is too thin to move in (a sliver a boolean left), and its hollow could not be made from its pieces",
        ));
    }
    if crossing_pairs(&inner) == 0 {
        // A copy that does not cross itself may still be inside out: a
        // ring hollowed thicker than its tube comes out through its own
        // middle, whole and the right way round. Its walls then face
        // against their own, and it is no hollow.
        if turned_share(s, &planes) > 0.5 {
            return Err(EditError::InsideOut { thickness: t });
        }
        return cut(&inner).map(|(out, r)| {
            let mut report = EditReport::default();
            report.add(&r);
            (out, report)
        });
    }
    // The inner solid folds through itself where a face shrinks past
    // nothing (a chamfer, fillet or rounded corner narrower than the wall,
    // `docs/research.md` §27): its outside is cut instead (`self_union`),
    // if that result is closed (an inner solid touching the outer along an
    // edge would leave a wall of no thickness there). Otherwise, or where
    // it crosses itself more, the folded solid is cut as before. The
    // boolean counts only pairs at corners it made, so here the result's
    // own pairs are counted.
    let counted = |out: Solid| {
        let crossing_pairs = crossing_pairs(&out);
        (out, EditReport { crossing_pairs })
    };
    let untangled = self_union(&inner)
        .ok()
        .and_then(|u| cut(&u).ok())
        .filter(|(_, r)| r.touching_edges == 0)
        .map(|(out, _)| counted(out));
    let untangled = match untangled {
        Some(u) if u.1.crossing_pairs == 0 => return Ok(u),
        u => u,
    };
    if let Some(i) = &exact {
        let e = cut(i)
            .ok()
            .filter(|(_, r)| r.touching_edges == 0)
            .map(|(out, _)| counted(out));
        if let Some(e) = e
            && e.1.crossing_pairs == 0
        {
            return Ok(e);
        }
    }
    // The faces that turned over taken out, their neighbours run on
    // (`collapse`): no convexity needed.
    let collapsed =
        collapse::collapsed(s, &targets, &[], size(s)).filter(|i| crossing_pairs(i) == 0);
    if let Some(i) = &collapsed {
        let e = cut(i)
            .ok()
            .filter(|(_, r)| r.touching_edges == 0)
            .map(|(out, _)| counted(out));
        if let Some(e) = e
            && e.1.crossing_pairs == 0
        {
            return Ok(e);
        }
    }
    // A hollow that pinches off: the solid taken apart at its flat faces
    // with holes, each piece's inner solid found, the pockets' cut from
    // the body's (`pieces`).
    if let Some(i) = pieces::pieced(s, &targets, t, size(s)) {
        let e = cut(&i)
            .ok()
            .filter(|(_, r)| r.touching_edges == 0)
            .map(|(out, _)| counted(out));
        if let Some(e) = e
            && e.1.crossing_pairs == 0
        {
            return Ok(e);
        }
    }
    // A wall that is exactly some distance of the part (a blind hole
    // ending the wall's thickness short of another) puts faces of the
    // inner copy on each other, which no untangling decides. The same
    // shell a millionth thicker or thinner has no such coincidence:
    // every face moves as one, so the copy stays as smooth as it was,
    // and the walls are off by that share of their thickness.
    if again {
        for share in [1e-6, -1e-6, 1e-5, -1e-5] {
            let near = Shell {
                thickness: t * (1.0 + share),
                open: d.open.clone(),
            };
            if let Ok(e) = shell_built(s, &near, false)
                && e.1.crossing_pairs == 0
                && e.0.measure().closed
            {
                return Ok(e);
            }
        }
    }
    // Surfaces that touch without crossing (two holes of one diameter
    // meeting: their inner copies are tangent along a line) are a case
    // the untangling cannot decide. Each vertex's move is made a little
    // longer or shorter, by its number, up to a millionth, then a
    // hundred-thousandth, then a ten-thousandth of itself: tangent
    // surfaces then cross or miss, and the copy untangles. The walls
    // are off by that share of their thickness.
    for share in [1e-6, 1e-5, 1e-4] {
        let mut nudged = Solid::default();
        for (v, p) in s.positions.iter().enumerate() {
            // A fixed scramble of the vertex's number, in −1..1.
            let h =
                ((v as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as f64 / (1u64 << 24) as f64;
            nudged.push_vertex(*p + offset(&planes[v]) * (1.0 + share * (2.0 * h - 1.0)));
        }
        for (i, tri) in s.triangles.iter().enumerate() {
            nudged.push_triangle(*tri, s.surfaces[i], &["wall"]);
        }
        nudged.uvs.clone_from(&s.uvs);
        if nudged.canonicalize().is_err() {
            continue;
        }
        let e = self_union(&nudged)
            .ok()
            .filter(|u| {
                // A nudge can also untangle wrongly; then some of the
                // inner solid lies against the outside. What is kept
                // has a quarter of the wall everywhere.
                let least = crate::mesh::morph::distances(&[s], u)
                    .into_iter()
                    .fold(f64::INFINITY, f64::min);
                least >= 0.25 * t
            })
            .and_then(|u| cut(&u).ok())
            .filter(|(_, r)| r.touching_edges == 0)
            .map(|(out, _)| counted(out));
        if let Some(e) = e
            && e.1.crossing_pairs == 0
        {
            return Ok(e);
        }
    }
    match (untangled, cut(&inner).map(|(out, _)| counted(out))) {
        (Some(u), Ok(f)) if u.1.crossing_pairs < f.1.crossing_pairs => Ok(u),
        (Some(u), Err(_)) => Ok(u),
        (_, f) => f,
    }
}

/// The share of `s`'s area whose triangles face the other way once each
/// vertex is moved by its planes ([`offset`]): 0 for a copy that keeps
/// its shape, 1 for one turned inside out.
pub(crate) fn turned_share(s: &Solid, planes: &[Vec<(DVec3, f64)>]) -> f64 {
    let moved: Vec<DVec3> = s
        .positions
        .iter()
        .enumerate()
        .map(|(v, p)| *p + offset(&planes[v]))
        .collect();
    let (mut turned, mut whole) = (0.0, 0.0);
    for tri in &s.triangles {
        let [a, b, c] = tri.map(|v| s.positions[v as usize]);
        let [a1, b1, c1] = tri.map(|v| moved[v as usize]);
        let n = (b - a).cross(c - a);
        whole += n.length();
        if (b1 - a1).cross(c1 - a1).dot(n) < 0.0 {
            turned += n.length();
        }
    }
    if whole > 0.0 { turned / whole } else { 0.0 }
}
