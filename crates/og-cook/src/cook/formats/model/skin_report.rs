//! Skin diagnostics and explicit weight relaxation, before welding/LOD generation. Triangle IDs
//! are source primitive index triples, not the reordered cooked triangles.
//! Probes rotate about bind joints in model axes (+Y up, +Z forward), composed
//! down the hierarchy. They are stress tests, not animations.

use super::{GltfSource, Mat4, Path, Quat, Vec3, node_order};
use og_anim::{HumanoidMap, NONE, SkeletonAsset, SkeletonNode, Trs};
use std::collections::BTreeMap;

mod relax;

type Weights = (Vec<[u16; 4]>, Vec<[f32; 4]>);
type Repairs = BTreeMap<(usize, usize), Weights>;

struct Binding {
    skin: usize,
    roots: Vec<usize>,
    parents: Vec<Option<usize>>,
    region: Vec<Option<usize>>,
    poses: Vec<Pose>,
}

struct Pose {
    name: String,
    bind: Mat4,
    palette: Vec<Mat4>,
}

struct Probe {
    name: String,
    rotations: Vec<(usize, f32)>,
}

fn topology(parents: &[Option<usize>]) -> (Vec<usize>, Vec<usize>) {
    let mut children = vec![Vec::new(); parents.len()];
    for (i, p) in parents.iter().enumerate() {
        if let Some(p) = p {
            children[*p].push(i);
        }
    }
    fn depth(i: usize, children: &[Vec<usize>], depths: &mut [usize]) -> usize {
        depths[i] = 1 + children[i]
            .iter()
            .map(|&c| depth(c, children, depths))
            .max()
            .unwrap_or(0);
        depths[i]
    }
    let mut depths = vec![0; parents.len()];
    let mut regions = Vec::new();
    let mut probes = Vec::new();
    for root in (0..parents.len()).filter(|&i| parents[i].is_none()) {
        depth(root, &children, &mut depths);
        let mut spine = root;
        loop {
            // Earliest joint breaks equal-length ties, independent of names.
            let next = children[spine]
                .iter()
                .copied()
                .max_by_key(|&c| (depths[c], std::cmp::Reverse(c)));
            let Some(next) = next else { break };
            let branches: Vec<_> = children[spine]
                .iter()
                .copied()
                .filter(|&c| c != next && depths[c] >= 3)
                .collect();
            if !branches.is_empty() {
                probes.extend(branches.iter().copied());
                probes.push(next);
                regions.extend(branches);
            }
            spine = next;
        }
    }
    probes.sort_unstable();
    probes.dedup();
    (regions, probes)
}

fn regions(parents: &[Option<usize>], roots: &[usize]) -> Vec<Option<usize>> {
    (0..parents.len())
        .map(|i| {
            let mut node = Some(i);
            while let Some(i) = node {
                if let Some(r) = roots.iter().position(|&r| r == i) {
                    return Some(r);
                }
                node = parents[i];
            }
            None
        })
        .collect()
}

fn probes(
    map: &HumanoidMap,
    joints: &[usize],
    parents: &[Option<usize>],
) -> (Vec<usize>, Vec<Probe>) {
    let roots: Option<Vec<usize>> = [
        "leftUpperArm",
        "rightUpperArm",
        "leftUpperLeg",
        "rightUpperLeg",
    ]
    .iter()
    .map(|r| {
        map.node(r)
            .and_then(|n| joints.iter().position(|&j| j == n as usize))
    })
    .collect();
    if let Some(roots) = roots {
        let poses: [(&str, &[(&str, f32)]); 3] = [
            (
                "rightArmForward",
                &[
                    ("rightUpperArm", -70.0),
                    ("rightLowerArm", -40.0),
                    ("leftUpperArm", 10.0),
                ],
            ),
            (
                "leftArmForward",
                &[
                    ("leftUpperArm", -70.0),
                    ("leftLowerArm", -40.0),
                    ("rightUpperArm", 10.0),
                ],
            ),
            (
                "stride",
                &[
                    ("leftUpperLeg", -35.0),
                    ("leftLowerLeg", 30.0),
                    ("rightUpperLeg", 25.0),
                ],
            ),
        ];
        return (
            roots,
            poses
                .into_iter()
                .map(|(name, rotations)| Probe {
                    name: name.into(),
                    rotations: rotations
                        .iter()
                        .filter_map(|(bone, angle)| map.node(bone).map(|j| (j as usize, *angle)))
                        .collect(),
                })
                .collect(),
        );
    }
    let (roots, branches) = topology(parents);
    (
        roots,
        branches
            .into_iter()
            .map(|j| Probe {
                name: format!("joint{j}Forward"),
                rotations: vec![(joints[j], -70.0)],
            })
            .collect(),
    )
}

fn longest(points: &[Vec3], t: &[u32]) -> f32 {
    let [a, b, c] = [
        points[t[0] as usize],
        points[t[1] as usize],
        points[t[2] as usize],
    ];
    a.distance(b).max(b.distance(c)).max(c.distance(a))
}

/// Notes include informational counts and per-pose triangle IDs. Only a
/// stretch note starts with `warning:`. Recomputed on incremental cooks too.
pub(in crate::cook) fn report(
    src: &GltfSource,
    root: &Path,
    rig: Option<&str>,
) -> Result<Vec<String>, String> {
    let (mut notes, _) = inspect(src, root, rig, false)?;
    notes.extend(src.skin_relax_notes.iter().cloned());
    Ok(notes)
}

/// Replace only repaired primitives' attributes, with private accessors so
/// shared/interleaved/normalized/sparse source accessors remain untouched.
pub(in crate::cook) fn repair(
    src: &mut GltfSource,
    root: &Path,
    rig: Option<&str>,
) -> Result<(), String> {
    let (notes, repairs) = inspect(src, root, rig, true)?;
    for ((mesh, prim), (joints, weights)) in repairs {
        for (semantic, component, bytes) in [
            ("JOINTS_0", 5123, bytemuck::cast_slice(&joints).to_vec()),
            ("WEIGHTS_0", 5126, bytemuck::cast_slice(&weights).to_vec()),
        ] {
            let buffer = src.buffers.len();
            let views = src.json["bufferViews"].as_array_mut().unwrap();
            let view = views.len();
            views.push(serde_json::json!({"buffer":buffer,"byteLength":bytes.len()}));
            src.json["buffers"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({"byteLength":bytes.len()}));
            src.buffers.push(gltf::buffer::Data(bytes));
            let accessors = src.json["accessors"].as_array_mut().unwrap();
            let accessor = accessors.len();
            accessors.push(
                serde_json::json!({"bufferView":view,"componentType":component,
                "count":joints.len(),"type":"VEC4"}),
            );
            src.json["meshes"][mesh]["primitives"][prim]["attributes"][semantic] =
                serde_json::json!(accessor);
        }
    }
    src.doc = gltf::Document::from_json(
        serde_json::from_value(src.json.clone()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    src.skin_relax_notes
        .extend(notes.into_iter().filter(|n| n.starts_with("skin relax:")));
    Ok(())
}

fn inspect(
    src: &GltfSource,
    root: &Path,
    rig: Option<&str>,
    repair: bool,
) -> Result<(Vec<String>, Repairs), String> {
    if src.doc.skins().next().is_none() {
        return Ok((Vec::new(), BTreeMap::new()));
    }
    let order = node_order(&src.doc, &mut |_| (0, 0)).map_err(|e| e.0)?;
    let mut skeleton = SkeletonAsset::default();
    let mut nodes: Vec<_> = src.doc.nodes().collect();
    nodes.sort_by_key(|n| order.new_index[n.index()]);
    for (node, row) in nodes.iter().zip(&order.rows) {
        skeleton.nodes.push(SkeletonNode {
            name: node.name().unwrap_or("").into(),
            parent: row.parent,
            mesh: row.mesh,
            rest: Trs::new(
                Vec3::new(row.tx, row.ty, row.tz),
                Quat::from_xyzw(row.rx, row.ry, row.rz, row.rw),
                Vec3::new(row.sx, row.sy, row.sz),
            ),
        });
    }
    let map = crate::cook::anim::humanoid(root, rig, &skeleton)?;
    let mut notes = Vec::new();
    let mut repairs = BTreeMap::new();
    let mut bindings: BTreeMap<usize, Vec<Binding>> = BTreeMap::new();
    let binds = super::skin_binds(src);
    // A mesh can be instanced with different skins: diagnose each binding.
    let mut seen = std::collections::BTreeSet::new();
    for node in src.doc.nodes() {
        let (Some(mesh), Some(skin)) = (node.mesh(), node.skin()) else {
            continue;
        };
        if !seen.insert((mesh.index(), skin.index())) {
            continue;
        }
        let joints: Vec<_> = skin
            .joints()
            .map(|j| order.new_index[j.index()] as usize)
            .collect();
        let parents: Vec<_> = joints
            .iter()
            .map(|&j| {
                let mut p = skeleton.nodes[j].parent;
                while p != NONE {
                    if let Some(i) = joints.iter().position(|&j| j == p as usize) {
                        return Some(i);
                    }
                    p = skeleton.nodes[p as usize].parent;
                }
                None
            })
            .collect();
        let (roots, poses) = probes(&map, &joints, &parents);
        let region = regions(&parents, &roots);
        let inverse: Vec<_> = skin
            .reader(|b| Some(&src.buffers[b.index()].0))
            .read_inverse_bind_matrices()
            .map(|ms| ms.map(|m| Mat4::from_cols_array_2d(&m)).collect())
            .unwrap_or_else(|| vec![Mat4::IDENTITY; joints.len()]);
        if inverse.len() != joints.len() {
            return Err("inverse bind matrix count differs from joints".into());
        }
        let bind = binds.get(&node.index()).copied().unwrap_or(Mat4::IDENTITY);
        // The node's default TRS need not be its bind pose. The inverse bind
        // locates each pivot in accessor space; the skin bind carries that
        // space into model units (including gltfpack's dequantisation).
        let mut pivots = vec![Vec3::ZERO; skeleton.nodes.len()];
        for (&j, ib) in joints.iter().zip(&inverse) {
            pivots[j] = bind.transform_point3(ib.inverse().transform_point3(Vec3::ZERO));
        }
        let poses = poses
            .into_iter()
            .map(|pose| {
                let mut delta = vec![Mat4::IDENTITY; skeleton.nodes.len()];
                for (i, n) in skeleton.nodes.iter().enumerate() {
                    let angle = pose
                        .rotations
                        .iter()
                        .find(|(j, _)| *j == i)
                        .map_or(0.0, |(_, a)| *a);
                    let pivot = pivots[i];
                    let local = Mat4::from_translation(pivot)
                        * Mat4::from_rotation_x(angle.to_radians())
                        * Mat4::from_translation(-pivot);
                    delta[i] = if n.parent == NONE {
                        local
                    } else {
                        delta[n.parent as usize] * local
                    };
                }
                Pose {
                    name: pose.name,
                    bind,
                    palette: joints.iter().map(|&j| delta[j] * bind).collect(),
                }
            })
            .collect();
        bindings.entry(mesh.index()).or_default().push(Binding {
            skin: skin.index(),
            roots,
            parents,
            region,
            poses,
        });
    }
    for mesh in src.doc.meshes() {
        let Some(bindings) = bindings.get(&mesh.index()) else {
            continue;
        };
        for prim in mesh
            .primitives()
            .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
        {
            let reader = prim.reader(|b| Some(&src.buffers[b.index()].0));
            let (Some(p), Some(j), Some(w)) = (
                reader.read_positions(),
                reader.read_joints(0),
                reader.read_weights(0),
            ) else {
                continue;
            };
            let positions: Vec<_> = p.map(Vec3::from).collect();
            let mut js: Vec<_> = j.into_u16().collect();
            let mut ws: Vec<_> = w.into_f32().collect();
            let indices: Vec<_> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());
            if js.len() != positions.len()
                || ws.len() != positions.len()
                || bindings
                    .iter()
                    .any(|b| js.iter().flatten().any(|&j| j as usize >= b.region.len()))
                || indices.iter().any(|&i| i as usize >= positions.len())
            {
                return Err("skin vertex attributes or indices out of range".into());
            }
            let palettes: Vec<_> = bindings.iter().flat_map(|b| &b.poses).collect();
            if repair {
                let changed = relax::relax(&positions, &indices, &palettes, &mut js, &mut ws);
                let remaining: usize = palettes
                    .iter()
                    .map(|p| stretched(&positions, &indices, &js, &ws, p).len())
                    .sum();
                notes.push(format!("skin relax: mesh {} primitive {}: {changed} of {} vertices reweighted; {remaining} remaining stretched triangles across {} probe poses (limit 12 passes)", mesh.index(), prim.index(), positions.len(), palettes.len()));
                if changed > 0 {
                    repairs.insert((mesh.index(), prim.index()), (js.clone(), ws.clone()));
                }
            }
            for binding in bindings {
                let Binding {
                    roots,
                    parents,
                    region,
                    poses,
                    ..
                } = binding;
                let cross = js
                    .iter()
                    .zip(&ws)
                    .filter(|(js, ws)| {
                        let mut weights = vec![0.0; roots.len()];
                        for (&j, &w) in js.iter().zip(ws.iter()) {
                            if let Some(r) = region[j as usize] {
                                weights[r] += w;
                            }
                        }
                        let dominant = weights
                            .iter()
                            .enumerate()
                            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)));
                        dominant.is_some_and(|(d, &w)| {
                            w > 0.0
                                && weights.iter().enumerate().any(|(r, &w)| {
                                    r != d && w > 0.02 && parents[roots[r]] != parents[roots[d]]
                                })
                        })
                    })
                    .count();
                let label = format!(
                    "skin {} mesh {} '{}' primitive {}",
                    binding.skin,
                    mesh.index(),
                    mesh.name().unwrap_or(""),
                    prim.index()
                );
                notes.push(format!("skin diagnostics: {label}: {} vertices, {cross} cross-limb vertices (information only), {} probe poses", positions.len(), poses.len()));
                for pose in poses {
                    let name = &pose.name;
                    let bad = stretched(&positions, &indices, &js, &ws, pose);
                    let severity = if bad.is_empty() {
                        "skin diagnostics:"
                    } else {
                        "warning: skin stretch:"
                    };
                    let hint = if bad.is_empty() {
                        ""
                    } else {
                        "; opt in to model { relaxSkin true } in import settings to relax stretched regions"
                    };
                    notes.push(format!("{severity} {label}: {name}: {} stretched triangles (longest edge >2.5x bind and >0.08 model units); source triangle IDs {bad:?}{hint}", bad.len()));
                }
            }
        }
    }
    Ok((notes, repairs))
}

fn stretched(
    positions: &[Vec3],
    indices: &[u32],
    js: &[[u16; 4]],
    ws: &[[f32; 4]],
    pose: &Pose,
) -> Vec<usize> {
    let bind: Vec<_> = positions
        .iter()
        .map(|&p| pose.bind.transform_point3(p))
        .collect();
    let posed: Vec<Vec3> = positions
        .iter()
        .zip(js)
        .zip(ws)
        .map(|((&p, js), ws)| {
            let sum: f32 = ws.iter().sum();
            if sum == 0.0 {
                return pose.bind.transform_point3(p);
            }
            js.iter()
                .zip(ws)
                .map(|(&j, &w)| pose.palette[j as usize].transform_point3(p) * (w / sum))
                .sum()
        })
        .collect();
    indices
        .as_chunks::<3>()
        .0
        .iter()
        .enumerate()
        .filter_map(|(id, t)| {
            let e0 = longest(&bind, t);
            let e1 = longest(&posed, t);
            (e1 > 0.08 && e1 > 2.5 * e0).then_some(id)
        })
        .collect()
}
