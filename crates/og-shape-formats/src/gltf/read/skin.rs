//! A glTF skin read onto a part: bones, inverse binds, joints and weights, the armature's uniform scale taken out.

use super::*;

/// The space a skin was read into: what a clip needs to follow it.
pub(crate) struct SkinSpace {
    /// Each bone's node in the file, in the skin's bone order.
    pub(crate) nodes: Vec<usize>,
    /// Each bone's inverse bind matrix as the file has it (glTF space).
    pub(crate) ibms: Vec<Affine>,
    /// Kernel space: from where the solid's vertices stand to where the
    /// file's own mesh coordinates would put them (the identity unless a
    /// scale was taken out of the bones, which moved the vertices to
    /// their bind pose).
    pub(crate) unbind: Affine,
}

impl Doc<'_> {
    /// Skin `k` for a mesh whose vertices (`positions`, kernel space) carry
    /// `raw` (joint, weight) pairs: its joints as bones, parents first, with
    /// their world rest frames and the file's inverse bind matrices, in
    /// kernel space; the weights through `Weights::from_raw`.
    ///
    /// Bones rigid up to one uniform scale (a scaled armature) have it
    /// taken out: the rest frames are the world transforms divided by it,
    /// the inverse bind matrices their inverses, and the vertices are
    /// returned where the file's bind pose puts them, with the scale and
    /// how well the file's matrices agree. A scale that
    /// is not uniform, differs between bones, or a skin that does not
    /// check, is an error.
    pub(crate) fn skin(
        &self,
        k: usize,
        name: &str,
        raw: &[Vec<(u32, f64)>],
        positions: &[DVec3],
    ) -> Result<(ReadSkin, SkinSpace), ImportError> {
        let skin = self.json.get("skins").at(k);
        let joints: Vec<usize> = skin
            .get("joints")
            .items()
            .iter()
            .map(|j| {
                j.index()
                    .ok_or_else(|| fmt_err(format!("{name}: a skin joint is not a node")))
            })
            .collect::<Result<_, _>>()?;
        if joints.is_empty()
            || joints
                .iter()
                .any(|&j| self.json.get("nodes").at(j).is_null())
        {
            return Err(fmt_err(format!(
                "{name}: its skin names no or missing joints"
            )));
        }
        let ibm: Option<Vec<f64>> = match skin.get("inverseBindMatrices").index() {
            Some(a) => {
                let (m, w) = self.accessor(a)?;
                if w != 16 || m.len() / 16 != joints.len() {
                    return Err(fmt_err(format!(
                        "{name}: its inverse bind matrices do not match its joints"
                    )));
                }
                Some(m)
            }
            None => None,
        };
        let parents = self.parents();
        // The bone each joint's nearest jointed ancestor is, and depth.
        let joint_of = |node: usize| joints.iter().position(|&j| j == node);
        let mut info: Vec<(usize, Option<usize>, usize)> = Vec::new();
        for (ji, &node) in joints.iter().enumerate() {
            let (mut at, mut depth, mut parent) = (node, 0, None);
            while let Some(p) = parents[at] {
                depth += 1;
                if parent.is_none() {
                    parent = joint_of(p);
                }
                at = p;
                if depth > parents.len() {
                    return Err(fmt_err(format!("{name}: the node tree loops")));
                }
            }
            info.push((depth, parent, ji));
        }
        // Parents first: by depth, then file order.
        let mut order: Vec<usize> = (0..joints.len()).collect();
        order.sort_by_key(|&ji| (info[ji].0, ji));
        let mut bone_of = vec![0u32; joints.len()];
        for (b, &ji) in order.iter().enumerate() {
            bone_of[ji] = b as u32;
        }
        let names: Vec<String> = {
            let mut used = BTreeMap::new();
            order
                .iter()
                .map(|&ji| {
                    let node = joints[ji];
                    unique(
                        &self
                            .json
                            .get("nodes")
                            .at(node)
                            .get("name")
                            .str()
                            .map_or_else(|| format!("node{node}"), str::to_string),
                        &mut used,
                    )
                })
                .collect()
        };
        // Each joint's world transform (glTF space) as a uniform scale
        // times a rigid motion; one scale for all, or none of this.
        let mut worlds: Vec<(f64, Affine, f64)> = Vec::with_capacity(order.len());
        for (b, &ji) in order.iter().enumerate() {
            let w = self.world(joints[ji], &parents);
            let sim = similarity(&w).ok_or_else(|| {
                ImportError::Unsupported(format!(
                    "{name}: bone {:?} is scaled unevenly or sheared (beyond {UNEVEN}); a bone must be rigid up to one uniform scale",
                    names[b]
                ))
            })?;
            worlds.push(sim);
        }
        let scale = worlds[0].0;
        if let Some(b) = worlds
            .iter()
            .position(|(s, _, _)| (s - scale).abs() > UNEVEN * scale)
        {
            return Err(ImportError::Unsupported(format!(
                "{name}: bones {:?} and {:?} are scaled differently ({} and {})",
                names[0], names[b], scale, worlds[b].0
            )));
        }
        let ibm_of = |ji: usize, bone: &str| -> Result<Affine, ImportError> {
            Ok(match &ibm {
                Some(m) => {
                    let m = &m[ji * 16..ji * 16 + 16];
                    if m[3] != 0.0 || m[7] != 0.0 || m[11] != 0.0 || m[15] != 1.0 {
                        return Err(ImportError::Unsupported(format!(
                            "{name}: bone {bone:?}'s inverse bind matrix is projective"
                        )));
                    }
                    Affine {
                        x: DVec3::new(m[0], m[1], m[2]),
                        y: DVec3::new(m[4], m[5], m[6]),
                        z: DVec3::new(m[8], m[9], m[10]),
                        t: DVec3::new(m[12], m[13], m[14]),
                    }
                }
                // No matrices: each is the identity (glTF).
                None => Affine::IDENTITY,
            })
        };
        // Rigid already (every axis of unit length, to 1e-9; the
        // inverse bind matrices' too, to what 32-bit floats keep): kept
        // as it is. Inverse bind matrices that scale (a file whose
        // positions are whole numbers of a grid, restored there) go
        // the other way: the vertices to where the bind pose puts
        // them.
        let rigid = order.iter().enumerate().all(|(b, &ji)| {
            let w = self.world(joints[ji], &parents);
            [w.x, w.y, w.z]
                .iter()
                .all(|a| (a.length() - 1.0).abs() < 1e-9)
                && ibm_of(ji, &names[b]).is_ok_and(|m| {
                    [m.x, m.y, m.z]
                        .iter()
                        .all(|a| (a.length() - 1.0).abs() < 1e-5)
                })
        });
        let mut bones = Vec::with_capacity(joints.len());
        let mut scaled = None;
        let mut unbind = Affine::IDENTITY;
        if rigid {
            // Rest frames are the world transforms; the file's inverse bind
            // matrices are kept as given.
            for (b, &ji) in order.iter().enumerate() {
                let w = conjugate(&self.world(joints[ji], &parents), 1000.0);
                let a = conjugate(&ibm_of(ji, &names[b])?, 1000.0);
                bones.push(Bone {
                    name: names[b].clone(),
                    parent: info[ji].1.map(|p| bone_of[p]),
                    rest: Frame {
                        origin: w.t,
                        x: w.x,
                        y: w.y,
                        z: w.z,
                    },
                    inverse_bind: [
                        a.x.x, a.x.y, a.x.z, 0.0, a.y.x, a.y.y, a.y.z, 0.0, a.z.x, a.z.y, a.z.z,
                        0.0, a.t.x, a.t.y, a.t.z, 1.0,
                    ],
                });
            }
        } else {
            // The scale comes out of the bones. The bind pose is the file's
            // inverse bind matrices: with `bind` = `world · inverse bind` of
            // the bone that weights the most vertices (where the file puts
            // the mesh), each bone's rest is `bind · inverse bind⁻¹`, the
            // uniform scale divided out, and its new inverse bind matrix is
            // that rest frame's inverse; the vertices go to `bind · v`.
            let ibms: Vec<Affine> = order
                .iter()
                .enumerate()
                .map(|(b, &ji)| ibm_of(ji, &names[b]))
                .collect::<Result<_, _>>()?;
            let nodes: Vec<Affine> = order
                .iter()
                .map(|&ji| self.world(joints[ji], &parents))
                .collect();
            let mut uses = vec![0usize; order.len()];
            for pairs in raw {
                for &(j, w) in pairs {
                    if let Some(&b) = bone_of.get(j as usize).filter(|_| w > 0.0) {
                        uses[b as usize] += 1;
                    }
                }
            }
            let main = (0..order.len())
                .max_by(|&a, &b| uses[a].cmp(&uses[b]).then(b.cmp(&a)))
                .unwrap_or(0);
            let bind = nodes[main].then(&ibms[main]);
            if bind.determinant() <= 0.0 {
                return Err(ImportError::Unsupported(format!(
                    "{name}: its bind pose mirrors or flattens the mesh"
                )));
            }
            let mut uneven: f64 = 0.0;
            let mut rests = Vec::with_capacity(order.len());
            for (b, ibm) in ibms.iter().enumerate() {
                let inv = affine_inverse(ibm).ok_or_else(|| {
                    ImportError::Unsupported(format!(
                        "{name}: bone {:?}'s inverse bind matrix is singular",
                        names[b]
                    ))
                })?;
                let (sb, rigid, u) = similarity(&bind.then(&inv)).ok_or_else(|| {
                    ImportError::Unsupported(format!(
                        "{name}: bone {:?} is scaled unevenly or sheared at bind time (beyond {UNEVEN}); a bone must be rigid up to one uniform scale",
                        names[b]
                    ))
                })?;
                if (sb - scale).abs() > UNEVEN * scale {
                    return Err(ImportError::Unsupported(format!(
                        "{name}: bone {:?} is scaled {sb} at bind time, the others {scale}",
                        names[b]
                    )));
                }
                uneven = uneven.max(u);
                rests.push(conjugate(&rigid, 1000.0));
            }
            let kbind = conjugate(&bind, 1000.0);
            unbind = affine_inverse(&kbind).ok_or_else(|| {
                ImportError::Unsupported(format!("{name}: its bind pose flattens the mesh"))
            })?;
            let placed: Vec<DVec3> = positions.iter().map(|p| kbind.apply(*p)).collect();
            // How far the file's own nodes stand from that bind pose, at the
            // vertices each bone weights: 0 to rounding when the file is
            // saved at its bind pose; more where it is saved posed.
            let mut offset: f64 = 0.0;
            let at_nodes: Vec<Affine> = (0..order.len())
                .map(|b| conjugate(&nodes[b].then(&ibms[b]), 1000.0))
                .collect();
            for (p, pairs) in positions.iter().zip(raw) {
                let q = kbind.apply(*p);
                for &(j, w) in pairs {
                    if let Some(&b) = bone_of.get(j as usize).filter(|_| w > 0.0) {
                        offset = offset.max((at_nodes[b as usize].apply(*p) - q).length());
                    }
                }
            }
            // The recomputed inverse bind matrices against the file's, the
            // scale factored out: `rest · S(scale) · file's` should be `bind`.
            let mut error: f64 = 0.0;
            for (p, pairs) in positions.iter().zip(raw) {
                let q = kbind.apply(*p);
                for &(j, w) in pairs {
                    if let Some(&b) = bone_of.get(j as usize).filter(|_| w > 0.0) {
                        let b = b as usize;
                        let scaled_rest = Affine {
                            x: rests[b].x * scale,
                            y: rests[b].y * scale,
                            z: rests[b].z * scale,
                            t: rests[b].t,
                        };
                        let file = scaled_rest.then(&conjugate(&ibms[b], 1000.0));
                        error = error.max((file.apply(*p) - q).length());
                    }
                }
            }
            // A bone rests where its node stands in the file, its scale
            // divided out; its inverse bind matrix is its bind-time
            // frame's inverse. In a file saved at its bind pose the two
            // frames are one (to rounding), and the bind-time one is
            // kept; in a file saved posed they differ, and the rest is
            // the node's, so every mesh of the skeleton has the same
            // bones and the file's rest pose is `rest · inverse bind`.
            let at_bind = (0..order.len()).all(|b| {
                let (n, r) = (conjugate(&worlds[b].1, 1000.0), &rests[b]);
                (n.t - r.t).length() <= 1e-6 * r.t.length().max(1.0)
                    && [(n.x, r.x), (n.y, r.y), (n.z, r.z)]
                        .iter()
                        .all(|(a, b)| (*a - *b).length() <= 1e-9)
            });
            for (b, &ji) in order.iter().enumerate() {
                let (bind_rest, inverse_bind) = rest_and_inverse(&rests[b]);
                let rest = if at_bind {
                    bind_rest
                } else {
                    rest_and_inverse(&conjugate(&worlds[b].1, 1000.0)).0
                };
                bones.push(Bone {
                    name: names[b].clone(),
                    parent: info[ji].1.map(|p| bone_of[p]),
                    rest,
                    inverse_bind,
                });
            }
            scaled = Some((
                placed,
                SkinScale {
                    part: name.to_string(),
                    scale,
                    bind_error: error,
                    uneven,
                    node_offset: offset,
                },
            ));
        }
        let mut weights = Vec::with_capacity(raw.len());
        for pairs in raw {
            let mut mapped = Vec::with_capacity(4);
            for &(j, w) in pairs {
                if w > 0.0 {
                    let b = *bone_of.get(j as usize).ok_or_else(|| {
                        fmt_err(format!(
                            "{name}: a vertex names joint {j}, which the skin lacks"
                        ))
                    })?;
                    mapped.push((b, w));
                }
            }
            weights.push(Weights::from_raw(mapped));
        }
        let skin = Skin { bones, weights };
        skin.check(raw.len())
            .map_err(|e| ImportError::Unsupported(format!("{name}: its skin: {e}")))?;
        let space = SkinSpace {
            nodes: order.iter().map(|&ji| joints[ji]).collect(),
            ibms: order
                .iter()
                .enumerate()
                .map(|(b, &ji)| ibm_of(ji, &names[b]))
                .collect::<Result<_, _>>()?,
            unbind,
        };
        Ok(((skin, scaled), space))
    }
}
