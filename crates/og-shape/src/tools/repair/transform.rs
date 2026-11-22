//! Parts into one solid: their transforms applied, skeletons and clips merged, the parts joined.

use super::*;

/// The part's solid placed by its transform. A mirroring transform
/// (negative determinant) reverses every triangle so faces still point
/// out. Names are kept. On a skinned part the transform must be rigid: it
/// moves the vertices, the bones' rest frames, and their inverse bind
/// matrices together (`IBM · T⁻¹`); a scaled or mirroring one is
/// [`RepairError::Skinned`].
pub fn apply_transform(part: &ImportedPart) -> Result<Solid, RepairError> {
    let m = &part.transform;
    let Some(skin) = &part.solid.skin else {
        return Ok(transformed(&part.solid, m));
    };
    if *m == Affine::IDENTITY {
        return Ok(part.solid.clone());
    }
    let rigid = [m.x, m.y, m.z]
        .iter()
        .all(|a| (a.length() - 1.0).abs() < 1e-9)
        && m.x.dot(m.y).abs() < 1e-9
        && m.y.dot(m.z).abs() < 1e-9
        && m.x.dot(m.z).abs() < 1e-9
        && m.determinant() > 0.0;
    if !rigid {
        return Err(RepairError::Skinned(format!(
            "part {:?} is skinned and placed by a scaled or mirroring transform",
            part.name
        )));
    }
    let mut out = transformed(&part.solid, m);
    let inverse = crate::tools::import::rigid_inverse(m);
    let inv = affine_matrix(&inverse);
    out.skin = Some(Skin {
        bones: skin
            .bones
            .iter()
            .map(|b| Bone {
                name: b.name.clone(),
                parent: b.parent,
                rest: crate::math::Frame {
                    origin: m.apply(b.rest.origin),
                    x: m.rotate(b.rest.x),
                    y: m.rotate(b.rest.y),
                    z: m.rotate(b.rest.z),
                },
                inverse_bind: mat_mul(&b.inverse_bind, &inv),
            })
            .collect(),
        weights: skin.weights.clone(),
    });
    Ok(out)
}

/// A column-major 4×4 matrix of an affine map.
pub(crate) fn affine_matrix(a: &Affine) -> Mat4 {
    [
        a.x.x, a.x.y, a.x.z, 0.0, a.y.x, a.y.y, a.y.z, 0.0, a.z.x, a.z.y, a.z.z, 0.0, a.t.x, a.t.y,
        a.t.z, 1.0,
    ]
}

/// `a · b`, column-major.
pub(crate) fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            out[4 * c + r] = (0..4).map(|k| a[4 * k + r] * b[4 * c + k]).sum();
        }
    }
    out
}

/// A merged skeleton, and each skinned part's bone indices in it.
/// A merged skeleton, each skinned part's bone indices in it, and for a
/// part bound in another space than the first with its bones, the
/// matrix that takes its vertices there.
pub(crate) type Merged = (Vec<Bone>, Vec<Vec<u32>>, Vec<Option<Mat4>>);

/// The skeletons of `parts` merged by bone name, and each skinned part's
/// bone indices in it. Bones of one name must agree: the same parent's
/// name (or none, in a skin that does not hold the parent), rest frame
/// and inverse bind matrix to rounding (1e-6 mm, 1e-9 in the axes).
///
/// A part whose bones rest where those already merged do, and whose
/// inverse bind matrices differ from theirs by one matrix for all the
/// bones they share (two meshes skinned to one skeleton, each bound in
/// its own space), is not a conflict: `M = first's inverse bind⁻¹ · its
/// own` takes its vertices to the first's space, where the first's
/// matrices move them as its own did. Bones that rest elsewhere are
/// another skeleton (a part placed with its bones), and stay one.
pub(crate) fn merge_skeletons(parts: &[(&str, &Solid)]) -> Result<Option<Merged>, RepairError> {
    let skinned: Vec<&str> = parts
        .iter()
        .filter(|p| p.1.skin.is_some())
        .map(|p| p.0)
        .collect();
    if skinned.is_empty() {
        return Ok(None);
    }
    let bare: Vec<&str> = parts
        .iter()
        .filter(|p| p.1.skin.is_none())
        .map(|p| p.0)
        .collect();
    if !bare.is_empty() {
        return Err(RepairError::Skinned(format!(
            "parts {skinned:?} are skinned and {bare:?} are not; join them apart"
        )));
    }
    let mut bones: Vec<Bone> = Vec::new();
    let mut maps = Vec::new();
    let mut conflicts: Vec<String> = Vec::new();
    // `slack`: 1 for parts in one bind space; a part taken to another's
    // was moved by a matrix read from f32 inverse binds.
    let same_rest = |a: &Bone, b: &Bone, slack: f64| {
        let f = (&a.rest, &b.rest);
        (f.0.origin - f.1.origin).length() <= 1e-6 * slack * f.0.origin.length().max(1.0)
            && (f.0.x - f.1.x).length() <= 1e-9 * slack
            && (f.0.y - f.1.y).length() <= 1e-9 * slack
            && (f.0.z - f.1.z).length() <= 1e-9 * slack
    };
    let same_matrix = |a: &Mat4, b: &Mat4, slack: f64| {
        a.iter()
            .zip(b)
            .all(|(x, y)| (x - y).abs() <= slack * x.abs().max(1.0))
    };
    let identity = crate::mesh::skin::frame_matrix(&crate::math::Frame::IDENTITY);
    let mut fixes: Vec<Option<Mat4>> = vec![None; parts.len()];
    // The part with the most bones first: a skin may hold only some of
    // the skeleton (a visor skinned to the neck and head alone), and its
    // first bones then have no parent in it. Such a bone takes the parent
    // the fuller skin gives it; a bone with a parent on both sides must
    // name the same one.
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by_key(|&i| {
        let n = parts[i].1.skin.as_ref().map_or(0, |k| k.bones.len());
        (core::cmp::Reverse(n), i)
    });
    maps.resize(parts.len(), Vec::new());
    for at in order {
        let (name, s) = &parts[at];
        let k = s.skin.as_ref().unwrap_or_else(|| unreachable!("{name}"));
        let mut map = Vec::with_capacity(k.bones.len());
        // The one matrix from this part's bind space to that of the
        // bones it shares with those merged, if their inverse binds
        // differ by one.
        let mut fix: Option<Mat4> = None;
        let mut one_space = true;
        for b in &k.bones {
            let Some(had) = bones.iter().find(|x| x.name == b.name) else {
                continue;
            };
            // Only bones that stand at one place can be one skeleton
            // bound in two spaces; a skeleton moved is another.
            if !same_rest(had, b, 1.0) {
                one_space = false;
            }
            let m = crate::mesh::skin::affine_inverse(&had.inverse_bind)
                .map(|inv| mat_mul(&inv, &b.inverse_bind));
            match (m, &fix) {
                (Some(m), None) => fix = Some(m),
                (Some(m), Some(f)) if same_matrix(f, &m, 1e-5) => {}
                _ => one_space = false,
            }
        }
        let fix = fix.filter(|f| {
            // Not a mirror, and not the identity.
            let det = f[0] * (f[5] * f[10] - f[9] * f[6]) - f[4] * (f[1] * f[10] - f[9] * f[2])
                + f[8] * (f[1] * f[6] - f[5] * f[2]);
            one_space && det > 0.0 && !same_matrix(&identity, f, 1e-6)
        });
        let unfix = fix.and_then(|f| crate::mesh::skin::affine_inverse(&f));
        fixes[at] = fix.filter(|_| unfix.is_some());
        for b in &k.bones {
            // In the merged space, this part's inverse binds are theirs
            // undone of the matrix.
            let moved;
            let b = match &unfix {
                Some(u) if fixes[at].is_some() => {
                    moved = Bone {
                        inverse_bind: mat_mul(&b.inverse_bind, u),
                        ..b.clone()
                    };
                    &moved
                }
                _ => b,
            };
            let parent_name = b.parent.map(|p| k.bones[p as usize].name.as_str());
            let parent = b.parent.map(|p| map[p as usize]);
            match bones.iter().position(|x| x.name == b.name) {
                Some(i) => {
                    let had = bones[i].parent.map(|p| bones[p as usize].name.as_str());
                    let same_parent = match (had, parent_name) {
                        (_, None) => true,
                        (None, Some(_)) => {
                            // Known as a root so far: it may take this
                            // parent if that comes before it.
                            match parent {
                                Some(p) if (p as usize) < i => {
                                    bones[i].parent = Some(p);
                                    true
                                }
                                _ => false,
                            }
                        }
                        (Some(a), Some(b)) => a == b,
                    };
                    let slack = if fixes[at].is_some() { 1e4 } else { 1.0 };
                    if !same_parent
                        || !same_rest(&bones[i], b, slack)
                        || !same_matrix(&bones[i].inverse_bind, &b.inverse_bind, 1e-6 * slack)
                    {
                        conflicts.push(b.name.clone());
                    }
                    map.push(i as u32);
                }
                None => {
                    bones.push(Bone {
                        parent,
                        ..b.clone()
                    });
                    map.push(bones.len() as u32 - 1);
                }
            }
        }
        maps[at] = map;
    }
    if !conflicts.is_empty() {
        conflicts.sort();
        conflicts.dedup();
        return Err(RepairError::Skinned(format!(
            "bones {conflicts:?} differ between the parts (parent, rest frame or inverse bind)"
        )));
    }
    Ok(Some((bones, maps, fixes)))
}

/// The clips of [`join`]'s solid: the parts' clips on the joined
/// skeleton, one a name. A clip's times are those of the part with the
/// most bones that has it; each bone's matrices come from the first such
/// part that holds the bone (read at those times), and a bone no part
/// moves in that clip stays at rest. Empty when no part has a clip.
pub fn join_clips(parts: &[ImportedPart]) -> Result<Vec<crate::mesh::SkinClip>, RepairError> {
    if parts.iter().all(|p| p.clips.is_empty()) {
        return Ok(Vec::new());
    }
    let placed: Vec<Solid> = parts
        .iter()
        .map(apply_transform)
        .collect::<Result<_, _>>()?;
    let named: Vec<(&str, &Solid)> = parts.iter().map(|p| p.name.as_str()).zip(&placed).collect();
    let (bones, maps, fixes) = match merge_skeletons(&named)? {
        Some(merged) => merged,
        // No skin: the clips drive morph targets alone.
        None => (
            Vec::new(),
            vec![Vec::new(); parts.len()],
            vec![None; parts.len()],
        ),
    };
    // The joined solid's targets, in `Solid::append`'s order, and each
    // one's weight at rest.
    let mut targets: Vec<(&str, f64)> = Vec::new();
    for s in &placed {
        for t in s.morphs.iter().flat_map(|m| &m.targets) {
            if !targets.iter().any(|(n, _)| *n == t.name) {
                targets.push((&t.name, t.weight));
            }
        }
    }
    // Each part's clips where `join` puts the part.
    let mut moved: Vec<Vec<crate::mesh::SkinClip>> = Vec::with_capacity(parts.len());
    for p in parts {
        let t = &p.transform;
        let m = [
            t.x.x, t.x.y, t.x.z, 0.0, t.y.x, t.y.y, t.y.z, 0.0, t.z.x, t.z.y, t.z.z, 0.0, t.t.x,
            t.t.y, t.t.z, 1.0,
        ];
        let mut clips = Vec::with_capacity(p.clips.len());
        // A part taken to the merged bind space: its matrices take its
        // vertices back first.
        let back = fixes[moved.len()]
            .as_ref()
            .and_then(crate::mesh::skin::affine_inverse);
        for c in &p.clips {
            let mut c = c.moved(&m).ok_or_else(|| {
                RepairError::Skinned(format!(
                    "part {:?} is placed by a singular transform",
                    p.name
                ))
            })?;
            if let Some(back) = &back {
                for s in c.poses.iter_mut().flatten() {
                    *s = mat_mul(s, back);
                }
            }
            clips.push(c);
        }
        moved.push(clips);
    }
    // Fullest skin first, as the skeleton was merged.
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by_key(|&i| (core::cmp::Reverse(maps[i].len()), i));
    let mut names: Vec<&str> = Vec::new();
    for &i in &order {
        for c in &moved[i] {
            if !names.contains(&c.name.as_str()) {
                names.push(&c.name);
            }
        }
    }
    let identity = crate::mesh::skin::frame_matrix(&crate::math::Frame::IDENTITY);
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let of = |i: usize| moved[i].iter().find(|c| c.name == name);
        let Some(times) = order.iter().find_map(|&i| of(i)).map(|c| c.times.clone()) else {
            continue;
        };
        let mut poses = vec![vec![identity; bones.len()]; times.len()];
        let mut set = vec![false; bones.len()];
        for &i in &order {
            let Some(clip) = of(i) else { continue };
            for (k, &t) in times.iter().enumerate() {
                let Some(at) = clip.at(t) else { continue };
                for (local, &merged) in maps[i].iter().enumerate() {
                    if !set[merged as usize] {
                        poses[k][merged as usize] = at[local];
                    }
                }
            }
            for &merged in &maps[i] {
                set[merged as usize] = true;
            }
        }
        // Each target's weight from the first part whose clip drives it.
        let mut morphs = Vec::new();
        if order
            .iter()
            .any(|&i| of(i).is_some_and(|c| !c.morphs.is_empty()))
        {
            for &t in &times {
                let mut row: Vec<f64> = targets.iter().map(|(_, w)| *w).collect();
                for (slot, (target, _)) in targets.iter().enumerate() {
                    for &i in &order {
                        let local = placed[i]
                            .morphs
                            .as_ref()
                            .and_then(|m| m.targets.iter().position(|x| x.name == *target));
                        let at = of(i).and_then(|c| c.weights_at(t));
                        if let (Some(local), Some(at)) = (local, at)
                            && let Some(w) = at.get(local)
                        {
                            row[slot] = *w;
                            break;
                        }
                    }
                }
                morphs.push(row);
            }
        }
        out.push(crate::mesh::SkinClip {
            name: name.to_string(),
            times,
            poses,
            morphs,
        });
    }
    Ok(out)
}

pub(crate) fn transformed(s: &Solid, m: &Affine) -> Solid {
    let mut out = s.clone();
    out.map_positions(|p| m.apply(p), m.determinant() < 0.0);
    out
}

/// Every part placed by its transform and joined into one solid, in
/// canonical order (exactly coincident vertices are welded). Each part's
/// surfaces stay their own; faces keep their names, so each part stays
/// addressable by its name. Skinned parts' skeletons are merged by bone
/// name: bones of one name must agree (parent, rest frame, inverse bind
/// matrix), and skinned and unskinned parts are not joined; either is
/// [`RepairError::Skinned`], naming them.
pub fn join(parts: &[ImportedPart]) -> Result<Solid, RepairError> {
    let placed: Vec<Solid> = parts
        .iter()
        .map(apply_transform)
        .collect::<Result<_, _>>()?;
    let names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    joined(&names, placed)
}

/// [`join`], taking the parts' solids: each is left empty once it is
/// placed, so a caller done with them holds the joined solid and not
/// the parts beside it (one mesh less from here on, two less while
/// joining; `docs/research.md` §78). The result is the same. On an
/// error the parts placed so far have been taken.
pub fn join_owned(parts: &mut [ImportedPart]) -> Result<Solid, RepairError> {
    let mut placed = Vec::with_capacity(parts.len());
    for p in parts.iter_mut() {
        placed.push(apply_transform(p)?);
        p.solid = Solid::default();
    }
    let names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    joined(&names, placed)
}

/// The placed solids of parts of these names as one.
pub(crate) fn joined(names: &[&str], placed: Vec<Solid>) -> Result<Solid, RepairError> {
    let named: Vec<(&str, &Solid)> = names.iter().copied().zip(&placed).collect();
    let merged = merge_skeletons(&named)?;
    let mut out = Solid::default();
    for (i, s) in placed.into_iter().enumerate() {
        let mut s = s;
        if let (Some((bones, maps, fixes)), true) = (&merged, s.skin.is_some()) {
            let map = &maps[i];
            // To the merged bind space, where the merged inverse binds
            // move the vertices as the part's own did.
            if let Some(f) = fixes[i] {
                s.map_positions(
                    |p| {
                        DVec3::new(
                            f[0] * p.x + f[4] * p.y + f[8] * p.z + f[12],
                            f[1] * p.x + f[5] * p.y + f[9] * p.z + f[13],
                            f[2] * p.x + f[6] * p.y + f[10] * p.z + f[14],
                        )
                    },
                    false,
                );
            }
            let Some(k) = &s.skin else { continue };
            let weights = k
                .weights
                .iter()
                .map(|w| Weights::from_raw(w.pairs().map(|(b, x)| (map[b as usize], x))))
                .collect();
            s.skin = Some(Skin {
                bones: bones.clone(),
                weights,
            });
        }
        out.append(&s);
    }
    out.canonicalize()?;
    Ok(out)
}
