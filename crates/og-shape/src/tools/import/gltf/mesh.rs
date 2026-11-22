//! A glTF primitive read into a part's solid: positions, normals, UVs, colours, morph targets, Draco decoded.

use super::*;

impl Doc<'_> {
    pub(crate) fn part(
        &self,
        name: &str,
        mesh: &Value,
        world: &Affine,
        out: &mut Imported,
        (images, image_bytes): (&[ImageInfo], &[Option<Vec<u8>>]),
    ) -> Result<(ImportedPart, RawWeights), ImportError> {
        let mut solid = Solid::default();
        let mut uvs: Vec<[DVec2; 3]> = Vec::new();
        let mut all_uv = true;
        let mut any_uv = false;
        let mut materials: Vec<Option<usize>> = Vec::new();
        let mut raw: Vec<Vec<(u32, f64)>> = Vec::new();
        let mut morphs: Vec<Vec<DVec3>> = Vec::new();
        for (pi, prim) in mesh.get("primitives").items().iter().enumerate() {
            let mode = prim.get("mode").index().unwrap_or(4);
            if !(4..=6).contains(&mode) {
                out.skipped
                    .push(format!("{name}: primitive {pi} draws points or lines"));
                continue;
            }
            let attrs = prim.get("attributes");
            let Some(pa) = attrs.get("POSITION").index() else {
                return Err(fmt_err(format!("{name}: primitive {pi} has no POSITION")));
            };
            // A Draco-compressed primitive: its points and triangles are
            // in the extension's buffer view, not behind the accessors.
            let packed = prim.get("extensions").get("KHR_draco_mesh_compression");
            let draco = match packed.get("bufferView").index() {
                Some(bv) => {
                    let (bytes, _) = self.view(bv)?;
                    Some(
                        draco::decode(bytes)
                            .map_err(|e| fmt_err(format!("{name}: primitive {pi}: {e}")))?,
                    )
                }
                None => None,
            };
            let attr = |key: &str, accessor: usize| -> Result<(Vec<f64>, usize), ImportError> {
                let (Some(d), Some(id)) = (&draco, packed.get("attributes").get(key).index())
                else {
                    return self.accessor(accessor);
                };
                let a = d
                    .attributes
                    .iter()
                    .find(|a| a.id as usize == id && !a.values.is_empty())
                    .ok_or_else(|| {
                        fmt_err(format!(
                            "{name}: primitive {pi}: no {key} in its Draco data"
                        ))
                    })?;
                // Whole numbers the accessor calls normalized are scaled
                // as its component type scales them.
                let acc = self.json.get("accessors").at(accessor);
                let scale = match acc.get("componentType").index() {
                    _ if a.integer == 0 || !acc.get("normalized").bool().unwrap_or(false) => 1.0,
                    Some(5120) => 127.0,
                    Some(5121) => 255.0,
                    Some(5122) => 32767.0,
                    Some(5123) => 65535.0,
                    _ => 1.0,
                };
                let values = a.values.iter().map(|v| (v / scale).max(-1.0)).collect();
                Ok((
                    if scale == 1.0 {
                        a.values.clone()
                    } else {
                        values
                    },
                    a.components,
                ))
            };
            let (pos, w) = attr("POSITION", pa)?;
            if draco.as_ref().is_some_and(|d| d.points * 3 != pos.len()) {
                return Err(fmt_err(format!(
                    "{name}: primitive {pi}: its Draco data has no position a point"
                )));
            }
            if w != 3 {
                return Err(fmt_err(format!("{name}: POSITION is not VEC3")));
            }
            let count = pos.len() / 3;
            // Morph targets: per target, each vertex's offset. Every
            // primitive of a mesh has the same targets; one with none
            // moves nothing.
            let targets = prim.get("targets").items();
            if morphs.is_empty() && !targets.is_empty() {
                morphs = vec![vec![DVec3::ZERO; solid.positions.len()]; targets.len()];
            }
            if !targets.is_empty() && targets.len() != morphs.len() {
                return Err(fmt_err(format!(
                    "{name}: primitive {pi} has {} morph targets, the others {}",
                    targets.len(),
                    morphs.len()
                )));
            }
            for (k, into) in morphs.iter_mut().enumerate() {
                match targets.get(k).and_then(|t| t.get("POSITION").index()) {
                    Some(a) => {
                        let (d, w) = self.accessor(a)?;
                        if w != 3 || d.len() / 3 != count {
                            return Err(fmt_err(format!(
                                "{name}: morph target {k} does not match POSITION"
                            )));
                        }
                        if d.iter().any(|x| !x.is_finite()) {
                            return Err(ImportError::NotFinite(name.to_string()));
                        }
                        into.extend(
                            d.as_chunks::<3>()
                                .0
                                .iter()
                                .map(|v| from_y_up(DVec3::new(v[0], v[1], v[2])) * 1000.0),
                        );
                    }
                    None => into.resize(into.len() + count, DVec3::ZERO),
                }
            }
            let uv = match attrs.get("TEXCOORD_0").index() {
                Some(ua) => {
                    let (uv, w) = attr("TEXCOORD_0", ua)?;
                    if w != 2 || uv.len() / 2 != count {
                        return Err(fmt_err(format!(
                            "{name}: TEXCOORD_0 does not match POSITION"
                        )));
                    }
                    Some(uv)
                }
                None => None,
            };
            let color = match attrs.get("COLOR_0").index() {
                Some(ca) => {
                    let (c, w) = attr("COLOR_0", ca)?;
                    if !(w == 3 || w == 4) || c.len() / w != count {
                        return Err(fmt_err(format!("{name}: COLOR_0 does not match POSITION")));
                    }
                    if c.iter().any(|x| !x.is_finite()) {
                        return Err(ImportError::NotFinite(name.to_string()));
                    }
                    let at = |v: usize| -> [f32; 4] {
                        let mut q = [1.0; 4];
                        q[..w].copy_from_slice(&c[v * w..v * w + w]);
                        crate::mesh::stored(q.map(|x| x.clamp(0.0, 1.0)))
                    };
                    Some((0..count).map(at).collect::<Vec<_>>())
                }
                None => None,
            };
            let index: Vec<u32> = match prim.get("indices").index() {
                _ if draco.is_some() => draco
                    .iter()
                    .flat_map(|d| d.faces.iter().flatten().copied())
                    .collect(),
                Some(ia) => {
                    let (idx, w) = self.accessor(ia)?;
                    if w != 1 {
                        return Err(fmt_err(format!("{name}: indices are not SCALAR")));
                    }
                    idx.iter().map(|&v| v as u32).collect()
                }
                None => (0..count as u32).collect(),
            };
            if index.iter().any(|&v| v as usize >= count) {
                return Err(fmt_err(format!("{name}: an index is out of range")));
            }
            let tris: Vec<[u32; 3]> = match mode {
                _ if draco.is_some() => index.as_chunks::<3>().0.to_vec(),
                4 => index.as_chunks::<3>().0.to_vec(),
                5 => (2..index.len())
                    .map(|k| {
                        if k % 2 == 0 {
                            [index[k - 2], index[k - 1], index[k]]
                        } else {
                            [index[k - 1], index[k - 2], index[k]]
                        }
                    })
                    .collect(),
                _ => (2..index.len())
                    .map(|k| [index[k - 1], index[k], index[0]])
                    .collect(),
            };
            let mi = prim.get("material").index();
            if !materials.contains(&mi) {
                materials.push(mi);
            }
            // Bone weights per vertex: (joint, weight) pairs, as the file has
            // them; empty where the primitive has none.
            match (
                attrs.get("JOINTS_0").index(),
                attrs.get("WEIGHTS_0").index(),
            ) {
                (Some(ja), Some(wa)) => {
                    let (j, jw) = attr("JOINTS_0", ja)?;
                    let (w, ww) = attr("WEIGHTS_0", wa)?;
                    if jw != 4 || ww != 4 || j.len() / 4 != count || w.len() / 4 != count {
                        return Err(fmt_err(format!(
                            "{name}: JOINTS_0 or WEIGHTS_0 does not match POSITION"
                        )));
                    }
                    for v in 0..count {
                        raw.push(
                            (0..4)
                                .map(|k| (j[v * 4 + k] as u32, w[v * 4 + k]))
                                .collect(),
                        );
                    }
                }
                _ => raw.extend((0..count).map(|_| Vec::new())),
            }
            let base = solid.positions.len() as u32;
            for v in pos.as_chunks::<3>().0 {
                let p = from_y_up(DVec3::new(v[0], v[1], v[2])) * 1000.0;
                if !p.is_finite() {
                    return Err(ImportError::NotFinite(name.to_string()));
                }
                solid.push_vertex(p);
            }
            for t in &tris {
                solid.push_triangle(t.map(|v| v + base), pi as u32, &[name]);
                if let Some(c) = &color {
                    let i = solid.triangles.len() - 1;
                    solid.set_colors(i, t.map(|v| c[v as usize]));
                }
                match &uv {
                    Some(uv) => {
                        any_uv = true;
                        // glTF's v runs down the image; the kernel's up.
                        uvs.push(
                            t.map(|v| DVec2::new(uv[v as usize * 2], 1.0 - uv[v as usize * 2 + 1])),
                        )
                    }
                    None => {
                        all_uv = false;
                        uvs.push([DVec2::ZERO; 3]);
                    }
                }
            }
        }
        if !morphs.is_empty() {
            // Names as exporters write them (`extras.targetNames`).
            let names = mesh.get("extras").get("targetNames").items();
            let rest = mesh.get("weights").nums().unwrap_or_default();
            let mut used = BTreeMap::new();
            solid.morphs = Some(crate::mesh::Morphs {
                targets: morphs
                    .into_iter()
                    .enumerate()
                    .map(|(k, deltas)| crate::mesh::Morph {
                        name: unique(
                            names
                                .get(k)
                                .and_then(|n| n.str())
                                .unwrap_or(&format!("target{k}")),
                            &mut used,
                        ),
                        weight: rest.get(k).copied().unwrap_or(0.0),
                        deltas,
                    })
                    .collect(),
            });
            // Vertices at one place are one vertex to every tool that
            // welds; where a target moves them apart (lips that touch
            // at rest), the weld takes the mean.
            let order = crate::math::order::sorted_order(&solid.positions);
            let mut remap = vec![0u32; solid.positions.len()];
            let mut groups = 0u32;
            for (k, &v) in order.iter().enumerate() {
                let p = solid.positions[v as usize];
                if k > 0
                    && crate::math::order::cmp_dvec3(solid.positions[order[k - 1] as usize], p)
                        .is_ne()
                {
                    groups += 1;
                }
                remap[v as usize] = groups;
            }
            let torn = solid
                .morphs
                .as_ref()
                .map_or(0, |m| m.torn(&remap, groups as usize + 1));
            if torn > 0 {
                out.skipped.push(format!(
                    "{name}: at {torn} places vertices that coincide move apart in a morph target; a tool that welds them moves them together (the mean)"
                ));
            }
        }
        if materials.len() > 1 {
            out.skipped.push(format!(
                "{name}: its primitives use {} materials; only the first is kept",
                materials.len()
            ));
        }
        let (material, defaults) = match materials.first() {
            Some(&mi) => self.material(name, mi, images, image_bytes, &mut out.skipped),
            None => (Material::default(), Vec::new()),
        };
        Ok((
            ImportedPart {
                clips: Vec::new(),
                name: name.to_string(),
                transform: conjugate(world, 1000.0),
                solid: Solid {
                    uvs: if all_uv && !uvs.is_empty() {
                        Some(uvs)
                    } else {
                        if any_uv {
                            out.skipped.push(format!(
                                "{name}: TEXCOORD_0 on some primitives only, so no UVs"
                            ));
                        }
                        None
                    },
                    ..solid
                },
                material,
                defaults,
            },
            raw,
        ))
    }
}
