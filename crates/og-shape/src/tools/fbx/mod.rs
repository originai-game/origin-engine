//! Import of FBX 7.x, binary and ASCII, into parts. Written here on
//! `miniz_oxide` (zlib for compressed arrays): ufbx is C, and the
//! pure-Rust `fbxcel` is a parser only, unmaintained since 2022
//! (`docs/research.md` §9).
//!
//! Read: meshes (`Geometry` with `Vertices` and `PolygonVertexIndex`;
//! polygons triangulated), the first UV layer, per-polygon materials as
//! surfaces, the model tree with FBX's full local transform (translation,
//! rotation offset and pivot, pre- and post-rotation, rotation in any
//! Euler order, scaling offset and pivot, scaling) and the geometric
//! transform, the file's axes and unit (`GlobalSettings`), and texture
//! images' sizes (embedded `Video` content or files through the
//! resolver), each part's first material ([`crate::tools::ImportedPart::material`]:
//! `DiffuseColor` × `DiffuseFactor` as its sRGB colour, `EmissiveColor`
//! × `EmissiveFactor`, `Opacity`, and the textures linked to
//! `DiffuseColor`, `NormalMap` and `EmissiveColor`; other linked textures
//! are listed as skipped), the first UV layer as `Solid::uvs`,
//! and skins: a `Skin` deformer's clusters as bones, parents
//! first, their rest frames and inverse bind matrices from the clusters'
//! `TransformLink` (the bone at bind time) and `Transform` (the mesh
//! then, in the bone's space), their `Indexes` and `Weights` as
//! weights, on the part's `Solid::skin` in the mesh's own coordinates (a
//! bone scaled at bind time is refused), and blend shapes as morph
//! targets, and the first colour layer as `Solid::colors` (RGBA as
//! stored, taken as linear). Skipped and listed: NURBS and other
//! geometry. FBX 6 and
//! older are [`ImportError::Unsupported`].

use std::collections::BTreeMap;

use crate::math::{DVec2, DVec3, Frame, trig};
use crate::mesh::{Bone, Skin, Solid, Weights};

use crate::tools::export::Material;
use crate::tools::image::{ImageInfo, Texture};
use crate::tools::import::{
    Affine, ImportError, Imported, ImportedPart, Resolve, SkinScale, fetch, fmt_err, from_y_up,
    place, rest_and_inverse, similarity, unique,
};
use crate::tools::polygon::triangulate;

mod curves;
mod mesh;
mod nodes;
mod transforms;
pub(crate) use curves::*;
pub(crate) use mesh::*;
pub(crate) use nodes::*;
pub(crate) use transforms::*;

// ── binary ──────────────────────────────────────────────────────────────────

// ── ASCII ───────────────────────────────────────────────────────────────────

// ── the scene ───────────────────────────────────────────────────────────────

/// Reads an FBX 7.x file, binary or ASCII. See the module docs for what
/// is read.
pub fn import_fbx(bytes: &[u8], resolve: Resolve) -> Result<Imported, ImportError> {
    let root = if bytes.starts_with(b"Kaydara FBX Binary") {
        let (version, root) = parse_binary(bytes)?;
        if version < 7000 {
            return Err(ImportError::Unsupported(format!("FBX version {version}")));
        }
        root
    } else {
        let text = String::from_utf8_lossy(bytes);
        let root = parse_ascii(&text)?;
        let version = root
            .child("FBXHeaderExtension")
            .and_then(|h| h.child("FBXVersion"))
            .and_then(|v| v.props.first())
            .and_then(Prop::int)
            .unwrap_or(0);
        if version < 7000 {
            return Err(ImportError::Unsupported(format!(
                "FBX version {version} (only 7.x)"
            )));
        }
        root
    };
    let objects = root
        .child("Objects")
        .ok_or_else(|| fmt_err("FBX has no Objects"))?;
    let mut by_id: BTreeMap<i64, &Node> = BTreeMap::new();
    for o in &objects.children {
        if let Some(id) = o.props.first().and_then(Prop::int) {
            by_id.insert(id, o);
        }
    }
    // Connections: object → object, in file order.
    let mut children_of: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut parent_of: BTreeMap<i64, i64> = BTreeMap::new();
    // Material → its textures, by the property each OP link names
    // (`DiffuseColor`, `NormalMap`, …), first link of each.
    let mut textures_of: BTreeMap<i64, BTreeMap<String, i64>> = BTreeMap::new();
    // Every property link: (child, parent, the parent's property).
    let mut op_links: Vec<(i64, i64, String)> = Vec::new();
    if let Some(c) = root.child("Connections") {
        for link in c.all("C") {
            let kind = link.props.first().and_then(Prop::str).unwrap_or("");
            let (Some(child), Some(parent)) = (
                link.props.get(1).and_then(Prop::int),
                link.props.get(2).and_then(Prop::int),
            ) else {
                continue;
            };
            if kind == "OP"
                && let Some(prop) = link.props.get(3).and_then(Prop::str)
            {
                textures_of
                    .entry(parent)
                    .or_default()
                    .entry(prop.to_string())
                    .or_insert(child);
                op_links.push((child, parent, prop.to_string()));
            }
            if kind == "OO" || kind == "OP" {
                children_of.entry(parent).or_default().push(child);
                let is_model = |id: i64| by_id.get(&id).is_some_and(|n| n.name == "Model");
                // A bone is also linked under its cluster; only a model (or
                // the root, 0) is a parent in the tree.
                if is_model(child) && (parent == 0 || is_model(parent)) {
                    parent_of.entry(child).or_insert(parent);
                }
            }
        }
    }
    let unit = root
        .child("GlobalSettings")
        .and_then(|g| g.p70_num("UnitScaleFactor"))
        .filter(|v| *v > 0.0 && v.is_finite())
        .unwrap_or(1.0)
        * 10.0;
    let ax = axes(&root);
    // File space to kernel space, unit-free: a signed permutation.
    let to_kernel = |v: DVec3| {
        let a = [v.x, v.y, v.z];
        from_y_up(DVec3::new(
            a[ax[0].0] * ax[0].1,
            a[ax[1].0] * ax[1].1,
            a[ax[2].0] * ax[2].1,
        ))
    };
    let p_det = {
        let e = |k: usize| {
            let mut v = DVec3::ZERO;
            v[k] = 1.0;
            to_kernel(v)
        };
        e(0).dot(e(1).cross(e(2)))
    };
    let conj = |m: &Affine| Affine {
        // P M P⁻¹ with P a signed permutation: P⁻¹ = Pᵀ.
        x: to_kernel(m.rotate(back(&to_kernel, DVec3::X))),
        y: to_kernel(m.rotate(back(&to_kernel, DVec3::Y))),
        z: to_kernel(m.rotate(back(&to_kernel, DVec3::Z))),
        t: to_kernel(m.t) * unit,
    };
    let mut out = Imported::default();
    let mut used = BTreeMap::new();
    let mut shapes = 0;
    for o in &objects.children {
        let class = o.props.get(2).and_then(Prop::str).unwrap_or("");
        if o.name == "Deformer"
            && !["Skin", "Cluster", "BlendShape", "BlendShapeChannel"].contains(&class)
        {
            shapes += 1;
        }
        if o.name == "Geometry" && !["Mesh", "Shape"].contains(&class) {
            out.skipped.push(format!(
                "geometry {:?} of kind {:?}",
                object_name(o.props.get(1).and_then(Prop::str).unwrap_or("")),
                o.props.get(2).and_then(Prop::str).unwrap_or("")
            ));
        }
    }
    if shapes > 0 {
        out.skipped.push(format!(
            "{shapes} deformers that are neither skins nor blend shapes: meshes read without them"
        ));
    }
    // A geometry's blend shapes: per channel a target (and how many
    // shapes the channel has), and the channels' ids in that order.
    let shapes_of = |g: &Node,
                     turned: &dyn Fn(DVec3) -> DVec3,
                     vertices: usize|
     -> (Vec<crate::mesh::Morph>, Vec<(i64, Vec<f64>)>) {
        let class = |n: &Node| n.props.get(2).and_then(Prop::str).unwrap_or("").to_string();
        let id_of = |n: &Node| n.props.first().and_then(Prop::int);
        let under = |id: Option<i64>, name: &'static str, kind: &'static str| -> Vec<&Node> {
            id.and_then(|id| children_of.get(&id))
                .into_iter()
                .flatten()
                .filter_map(|c| by_id.get(c))
                .filter(|n| n.name == name && class(n) == kind)
                .copied()
                .collect()
        };
        let (mut targets, mut channels) = (Vec::new(), Vec::new());
        for deformer in under(id_of(g), "Deformer", "BlendShape") {
            for channel in under(id_of(deformer), "Deformer", "BlendShapeChannel") {
                let shapes = under(id_of(channel), "Geometry", "Shape");
                let Some(cid) = id_of(channel) else { continue };
                // Each shape with the weight it is whole at (`FullWeights`,
                // percent; one shape alone at 100), in rising order.
                let full = channel
                    .array("FullWeights")
                    .filter(|w| w.len() == shapes.len());
                let mut keyed: Vec<(f64, &Node)> = shapes
                    .iter()
                    .enumerate()
                    .map(|(k, s)| (full.map_or(100.0, |w| w[k]) / 100.0, *s))
                    .filter(|(w, _)| w.is_finite() && *w != 0.0)
                    .collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
                if keyed.is_empty() {
                    continue;
                }
                let name = object_name(channel.props.get(1).and_then(Prop::str).unwrap_or(""));
                let keys: Vec<f64> = keyed.iter().map(|k| k.0).collect();
                let rest = shape_weights(deform_percent(&channel) / 100.0, &keys);
                // The shape at the largest weight bears the channel's
                // name; the others, shapes on the way to it, their weight
                // after it.
                let top = keys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                for (k, (weight, shape)) in keyed.iter().enumerate() {
                    let mut deltas = vec![DVec3::ZERO; vertices];
                    let index = shape.array("Indexes").unwrap_or(&[]);
                    let offsets = shape.array("Vertices").unwrap_or(&[]);
                    for (i, d) in index.iter().zip(offsets.as_chunks::<3>().0) {
                        let d = turned(DVec3::new(d[0], d[1], d[2]));
                        if let Some(slot) = deltas.get_mut(*i as usize).filter(|_| d.is_finite()) {
                            *slot = d;
                        }
                    }
                    targets.push(crate::mesh::Morph {
                        name: if *weight == top {
                            name.clone()
                        } else {
                            // In percent, to a thousandth.
                            format!("{name}@{}", (weight * 100_000.0).round() / 1000.0)
                        },
                        weight: rest[k],
                        deltas,
                    });
                }
                channels.push((cid, keys));
            }
        }
        (targets, channels)
    };
    // A geometry's skin: its Skin deformer's clusters, each a bone, with
    // the bind matrices the clusters carry (`Transform`: the mesh at bind
    // time; `TransformLink`: the bone), in the mesh's own coordinates.
    let skin_of = |g: &Node,
                   name: &str,
                   positions: &[DVec3]|
     -> Result<Option<(FbxSkin, FbxSpace)>, ImportError> {
        let vertices = positions.len();
        let Some(gid) = g.props.first().and_then(Prop::int) else {
            return Ok(None);
        };
        let class = |n: &Node| n.props.get(2).and_then(Prop::str).unwrap_or("").to_string();
        let Some(skin) = children_of
            .get(&gid)
            .into_iter()
            .flatten()
            .filter_map(|c| by_id.get(c))
            .find(|n| n.name == "Deformer" && class(n) == "Skin")
        else {
            return Ok(None);
        };
        let unsupported =
            |why: String| ImportError::Unsupported(format!("{name}: its skin: {why}"));
        let sid = skin.props.first().and_then(Prop::int).unwrap_or(0);
        // (bone model id, name, cluster)
        let mut clusters: Vec<(i64, String, &Node)> = Vec::new();
        for c in children_of.get(&sid).into_iter().flatten() {
            let Some(cluster) = by_id.get(c).filter(|n| class(n) == "Cluster") else {
                continue;
            };
            let cid = cluster.props.first().and_then(Prop::int).unwrap_or(0);
            let Some(bone) = children_of
                .get(&cid)
                .into_iter()
                .flatten()
                .find(|b| by_id.get(b).is_some_and(|n| n.name == "Model"))
            else {
                return Err(unsupported("a cluster has no bone".into()));
            };
            let bname = object_name(by_id[bone].props.get(1).and_then(Prop::str).unwrap_or(""));
            clusters.push((*bone, bname, cluster));
        }
        if clusters.is_empty() {
            return Err(unsupported("no clusters".into()));
        }
        let matrix = |c: &Node, key: &str| -> Result<Affine, ImportError> {
            let m = c
                .array(key)
                .filter(|m| m.len() == 16)
                .ok_or_else(|| unsupported(format!("a cluster has no {key}")))?;
            Ok(conj(&Affine {
                x: DVec3::new(m[0], m[1], m[2]),
                y: DVec3::new(m[4], m[5], m[6]),
                z: DVec3::new(m[8], m[9], m[10]),
                t: DVec3::new(m[12], m[13], m[14]),
            }))
        };
        let rigid = |a: &Affine| {
            [a.x, a.y, a.z]
                .iter()
                .all(|v| (v.length() - 1.0).abs() < 1e-9)
                && a.x.dot(a.y).abs() < 1e-9
                && a.y.dot(a.z).abs() < 1e-9
                && a.x.dot(a.z).abs() < 1e-9
                && a.determinant() > 0.0
        };
        let inverse = |a: &Affine| {
            let r = transpose(a);
            Affine {
                t: -r.rotate(a.t),
                ..r
            }
        };
        // Parents first: by depth in the model tree, then cluster order.
        let ids: Vec<i64> = clusters.iter().map(|c| c.0).collect();
        let mut info: Vec<(usize, Option<usize>)> = Vec::new();
        for &id in &ids {
            let (mut at, mut depth, mut parent) = (id, 0usize, None);
            while let Some(&p) = parent_of.get(&at) {
                if p == 0 || depth > 512 {
                    break;
                }
                depth += 1;
                if parent.is_none() {
                    parent = ids.iter().position(|&x| x == p);
                }
                at = p;
            }
            info.push((depth, parent));
        }
        let mut order: Vec<usize> = (0..clusters.len()).collect();
        order.sort_by_key(|&k| (info[k].0, k));
        let mut bone_of = vec![0u32; clusters.len()];
        for (b, &k) in order.iter().enumerate() {
            bone_of[k] = b as u32;
        }
        let mut used = BTreeMap::new();
        let mut names = Vec::with_capacity(order.len());
        let mut binds = Vec::with_capacity(order.len());
        for &k in &order {
            let (_, bname, cluster) = &clusters[k];
            names.push(unique(bname, &mut used));
            // `TransformLink` is the bone in the world at bind time.
            // `Transform` is the mesh then, as the bone sees it (the bone's
            // inverse times the mesh in the world): a file whose mesh
            // stands at the origin has each cluster's `Transform` the
            // inverse of its `TransformLink`. So the mesh in the world is
            // their product.
            let link = matrix(cluster, "TransformLink")?;
            binds.push((link.then(&matrix(cluster, "Transform")?), link));
        }
        let mut bones = Vec::with_capacity(clusters.len());
        let mut scaled = None;
        if binds.iter().all(|(m, b)| rigid(m) && rigid(b)) {
            for (i, &k) in order.iter().enumerate() {
                let (mesh_bind, bone_bind) = &binds[i];
                // The bone at rest in the mesh's coordinates, and its inverse.
                let rest = inverse(mesh_bind).then(bone_bind);
                let ibm = inverse(&rest);
                bones.push(Bone {
                    name: names[i].clone(),
                    parent: info[k].1.map(|p| bone_of[p]),
                    rest: Frame {
                        origin: rest.t,
                        x: rest.x,
                        y: rest.y,
                        z: rest.z,
                    },
                    inverse_bind: [
                        ibm.x.x, ibm.x.y, ibm.x.z, 0.0, ibm.y.x, ibm.y.y, ibm.y.z, 0.0, ibm.z.x,
                        ibm.z.y, ibm.z.z, 0.0, ibm.t.x, ibm.t.y, ibm.t.z, 1.0,
                    ],
                });
            }
        } else {
            // A uniform scale on the bones comes out: the rest frames are the
            // bones' bind transforms divided by it, and the vertices go where
            // the mesh's bind transform puts them, in the world.
            let mut scale = None;
            for (i, (mesh_bind, bone_bind)) in binds.iter().enumerate() {
                let (sb, _, _) = similarity(bone_bind).ok_or_else(|| {
                    unsupported(format!(
                        "bone {:?} is scaled unevenly or sheared at bind time",
                        names[i]
                    ))
                })?;
                if similarity(mesh_bind).is_none() {
                    return Err(unsupported(
                        "the mesh is scaled unevenly or sheared at bind time".into(),
                    ));
                }
                let s0 = *scale.get_or_insert(sb);
                if (sb - s0).abs() > crate::tools::import::UNEVEN * s0 {
                    return Err(unsupported(format!(
                        "bones {:?} and {:?} are scaled differently ({s0} and {sb})",
                        names[0], names[i]
                    )));
                }
            }
            // Each vertex goes where its clusters' mesh bind transforms put
            // it, blended by weight; how far they disagree is the error.
            let mut raw: Vec<Vec<(u32, f64)>> = vec![Vec::new(); vertices];
            for (i, &k) in order.iter().enumerate() {
                let c = clusters[k].2;
                let idx = c.array("Indexes").unwrap_or(&[]);
                let w = c.array("Weights").unwrap_or(&[]);
                for (&v, &x) in idx.iter().zip(w) {
                    if let Some(r) = raw.get_mut(v as usize) {
                        r.push((i as u32, x));
                    }
                }
            }
            let (placed, error) = place(positions, &raw, &|b| binds.get(b as usize).map(|m| m.0))
                .map_err(unsupported)?;
            let size = placed
                .iter()
                .fold(0.0f64, |m, q| m.max(q.abs().max_element()));
            if error > 1e-4 * size.max(1.0) {
                return Err(unsupported(format!(
                    "its clusters place a vertex differently at bind time, by {error} mm"
                )));
            }
            let mut uneven: f64 = 0.0;
            for (i, &k) in order.iter().enumerate() {
                let (_, rigid_link, u) = similarity(&binds[i].1).unwrap_or((1.0, binds[i].1, 0.0));
                uneven = uneven.max(u);
                let (rest, inverse_bind) = rest_and_inverse(&rigid_link);
                bones.push(Bone {
                    name: names[i].clone(),
                    parent: info[k].1.map(|p| bone_of[p]),
                    rest,
                    inverse_bind,
                });
            }
            scaled = Some((
                placed,
                SkinScale {
                    part: name.to_string(),
                    scale: scale.unwrap_or(1.0),
                    bind_error: error,
                    uneven,
                    node_offset: 0.0,
                },
            ));
        }
        let mut raw: Vec<Vec<(u32, f64)>> = vec![Vec::new(); vertices];
        for (k, (_, _, cluster)) in clusters.iter().enumerate() {
            let idx = cluster.array("Indexes").unwrap_or(&[]);
            let w = cluster.array("Weights").unwrap_or(&[]);
            for (&i, &x) in idx.iter().zip(w) {
                let i = i as usize;
                if i >= vertices {
                    return Err(unsupported(
                        "a cluster names a vertex that does not exist".into(),
                    ));
                }
                raw[i].push((bone_of[k], x));
            }
        }
        let skin = Skin {
            bones,
            weights: raw.into_iter().map(Weights::from_raw).collect(),
        };
        skin.check(vertices)
            .map_err(|e| unsupported(e.to_string()))?;
        // What a clip needs to follow this skin. The solid stands in the
        // mesh's own coordinates (rigid bones) or where its bind pose puts
        // it in the world (a scale taken out).
        let in_world = scaled.is_some();
        let mut unlink = Vec::with_capacity(order.len());
        for (i, (_, bone_bind)) in binds.iter().enumerate() {
            unlink.push(
                crate::tools::import::affine_inverse(bone_bind).ok_or_else(|| {
                    unsupported(format!("bone {:?}'s bind transform is singular", names[i]))
                })?,
            );
        }
        let space = FbxSpace {
            ids: order.iter().map(|&k| clusters[k].0).collect(),
            unlink,
            mesh: binds
                .iter()
                .map(|b| if in_world { Affine::IDENTITY } else { b.0 })
                .collect(),
            out: if in_world {
                Affine::IDENTITY
            } else {
                inverse(&binds[0].0)
            },
        };
        Ok(Some(((skin, scaled), space)))
    };
    // World transform of a model, by its parents.
    let world = |id: i64| -> Affine {
        let mut m = Affine::IDENTITY;
        let mut at = id;
        for _ in 0..256 {
            let Some(node) = by_id.get(&at) else { break };
            m = local(node).then(&m);
            match parent_of.get(&at) {
                Some(&p) if p != 0 => at = p,
                _ => break,
            }
        }
        m
    };
    // The file's animation stacks as they move a skin: each sampled at
    // its curves' key times and sixty times a second between them. A
    // stack's first layer is read; a curve's easing between its keys
    // (`Curve`) is in the samples.
    let kids = |id: i64, name: &str| -> Vec<i64> {
        children_of
            .get(&id)
            .into_iter()
            .flatten()
            .copied()
            .filter(|c| by_id.get(c).is_some_and(|n| n.name == name))
            .collect()
    };
    // Per stack: its name and its layers, the first undermost.
    let mut stacks: Vec<(String, Vec<Layer>)> = Vec::new();
    for o in objects.all("AnimationStack") {
        let Some(sid) = o.props.first().and_then(Prop::int) else {
            continue;
        };
        let sname = object_name(o.props.get(1).and_then(Prop::str).unwrap_or(""));
        let mut layers = Vec::new();
        for lid in kids(sid, "AnimationLayer") {
            let first = layers.is_empty();
            let node = by_id.get(&lid);
            let int = |key: &str| {
                node.and_then(|n| n.p70(key))
                    .and_then(|p| p.first())
                    .and_then(Prop::int)
                    .unwrap_or(0)
            };
            let mut layer = Layer {
                mode: int("BlendMode"),
                weight: node
                    .and_then(|n| n.p70_num("Weight"))
                    .map_or(1.0, |w| (w / 100.0).clamp(0.0, 1.0)),
                turns: int("RotationAccumulationMode") == 0,
                multiplies: int("ScaleAccumulationMode") == 0,
                driven: BTreeMap::new(),
                shapes: BTreeMap::new(),
                weight_curve: None,
            };
            // The layer's own weight, where a curve drives it.
            for (cn, p, prop) in &op_links {
                if *p != lid || prop != "Weight" {
                    continue;
                }
                for (c, q, prop) in &op_links {
                    if q != cn || prop != "d|Weight" {
                        continue;
                    }
                    let Some(curve) = by_id.get(c).filter(|n| n.name == "AnimationCurve") else {
                        continue;
                    };
                    if let (Some(t), Some(v)) =
                        (curve.array("KeyTime"), curve.array("KeyValueFloat"))
                        && !t.is_empty()
                        && t.len() == v.len()
                        && t.windows(2).all(|w| w[1] >= w[0])
                    {
                        layer.weight_curve = Some(Curve::new(
                            t.iter().map(|x| x / 46_186_158_000.0).collect(),
                            v.to_vec(),
                            curve.array("KeyAttrFlags").unwrap_or(&[]),
                            curve.array("KeyAttrDataFloat").unwrap_or(&[]),
                            curve.array("KeyAttrRefCount").unwrap_or(&[]),
                        ));
                    }
                }
            }
            for cn in kids(lid, "AnimationCurveNode") {
                // A blend-shape channel's weight, in percent.
                let channel = op_links.iter().find_map(|(c, p, prop)| {
                    (*c == cn
                        && prop == "DeformPercent"
                        && by_id.get(p).is_some_and(|n| n.name == "Deformer"))
                    .then_some(*p)
                });
                if let Some(channel) = channel {
                    let own = by_id.get(&channel).map_or(0.0, deform_percent);
                    let rest = by_id
                        .get(&cn)
                        .and_then(|n| n.p70_num("d|DeformPercent"))
                        .unwrap_or(if first { own } else { 0.0 });
                    let mut found = None;
                    for (c, p, prop) in &op_links {
                        if *p != cn || prop != "d|DeformPercent" {
                            continue;
                        }
                        let Some(curve) = by_id.get(c).filter(|n| n.name == "AnimationCurve")
                        else {
                            continue;
                        };
                        let (Some(t), Some(v)) =
                            (curve.array("KeyTime"), curve.array("KeyValueFloat"))
                        else {
                            continue;
                        };
                        if t.is_empty() || t.len() != v.len() || t.windows(2).any(|w| w[1] < w[0]) {
                            return Err(fmt_err(format!(
                                "animation {sname:?}: a curve's times and values do not match or do not rise"
                            )));
                        }
                        found = Some(Curve::new(
                            t.iter().map(|x| x / 46_186_158_000.0).collect(),
                            v.to_vec(),
                            curve.array("KeyAttrFlags").unwrap_or(&[]),
                            curve.array("KeyAttrDataFloat").unwrap_or(&[]),
                            curve.array("KeyAttrRefCount").unwrap_or(&[]),
                        ));
                    }
                    layer.shapes.insert(channel, (found, rest));
                    continue;
                }
                // The model property this curve node drives.
                let Some((model, slot)) = op_links.iter().find_map(|(c, p, prop)| {
                    let slot = match prop.as_str() {
                        "Lcl Translation" => 0,
                        "Lcl Rotation" => 1,
                        "Lcl Scaling" => 2,
                        _ => return None,
                    };
                    (*c == cn && by_id.get(p).is_some_and(|n| n.name == "Model"))
                        .then_some((*p, slot))
                }) else {
                    continue;
                };
                let channel = &mut layer.driven.entry(model).or_default()[slot];
                channel.on = true;
                // What an axis without a curve has in this layer.
                for (axis, key) in ["d|X", "d|Y", "d|Z"].into_iter().enumerate() {
                    // The undermost layer, where the file says nothing:
                    // the model's own value.
                    let own = by_id.get(&model).filter(|_| first).map(|m| {
                        let key = ["Lcl Translation", "Lcl Rotation", "Lcl Scaling"][slot];
                        m.p70_vec(key, if slot == 2 { DVec3::ONE } else { DVec3::ZERO })[axis]
                    });
                    channel.rest[axis] = by_id
                        .get(&cn)
                        .and_then(|n| n.p70_num(key))
                        .or(own)
                        .unwrap_or(if slot == 2 { 1.0 } else { 0.0 });
                }
                for (c, p, prop) in &op_links {
                    let axis = match prop.as_str() {
                        "d|X" => 0,
                        "d|Y" => 1,
                        "d|Z" => 2,
                        _ => continue,
                    };
                    if *p != cn {
                        continue;
                    }
                    let Some(curve) = by_id.get(c).filter(|n| n.name == "AnimationCurve") else {
                        continue;
                    };
                    let (Some(t), Some(v)) = (curve.array("KeyTime"), curve.array("KeyValueFloat"))
                    else {
                        continue;
                    };
                    if t.is_empty() || t.len() != v.len() || t.windows(2).any(|w| w[1] < w[0]) {
                        return Err(fmt_err(format!(
                            "animation {sname:?}: a curve's times and values do not match or do not rise"
                        )));
                    }
                    // FBX time: 46 186 158 000 ticks a second.
                    let seconds: Vec<f64> = t.iter().map(|x| x / 46_186_158_000.0).collect();
                    layer.driven.entry(model).or_default()[slot].curves[axis] = Some(Curve::new(
                        seconds,
                        v.to_vec(),
                        curve.array("KeyAttrFlags").unwrap_or(&[]),
                        curve.array("KeyAttrDataFloat").unwrap_or(&[]),
                        curve.array("KeyAttrRefCount").unwrap_or(&[]),
                    ));
                }
            }
            layers.push(layer);
        }
        if layers
            .iter()
            .any(|l| !l.driven.is_empty() || !l.shapes.is_empty())
        {
            stacks.push((sname, layers));
        }
    }
    // `channels`: the part's blend-shape channels, in its targets' order,
    // each with the weights its shapes are whole at.
    let clips_of = |space: &FbxSpace,
                    channels: &[(i64, Vec<f64>)]|
     -> Result<Vec<crate::mesh::SkinClip>, ImportError> {
        let mut clips = Vec::new();
        let mut used = BTreeMap::new();
        for (sname, layers) in &stacks {
            let mut times: Vec<f64> = layers
                .iter()
                .flat_map(|l| l.driven.values())
                .flatten()
                .flat_map(|ch| ch.curves.iter().flatten())
                .chain(
                    layers
                        .iter()
                        .flat_map(|l| l.shapes.values())
                        .filter_map(|s| s.0.as_ref()),
                )
                .flat_map(|c| c.times.iter().copied())
                .collect();
            if times.is_empty() {
                continue;
            }
            let (start, end) = times
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &t| {
                    (lo.min(t), hi.max(t))
                });
            let steps = ((end - start) * crate::tools::import::SAMPLES_A_SECOND).ceil();
            if steps >= 1.0 && steps < crate::tools::import::MAX_KEYS as f64 {
                times.extend(
                    (1..steps as usize).map(|k| start + (end - start) * (k as f64 / steps)),
                );
            }
            times.sort_by(f64::total_cmp);
            times.dedup();
            if times.len() > crate::tools::import::MAX_KEYS {
                let n = crate::tools::import::MAX_KEYS;
                times = (0..n)
                    .map(|k| start + (end - start) * (k as f64 / (n - 1) as f64))
                    .collect();
            }
            let mut poses = Vec::with_capacity(times.len());
            for &time in &times {
                // A model's local transform at this time.
                let local_at = |id: i64, node: &Node| -> Affine {
                    if !layers.iter().any(|l| l.driven.contains_key(&id)) {
                        return local(node);
                    }
                    let order = rotation_order(node);
                    let mut at = Layered {
                        t: node.p70_vec("Lcl Translation", DVec3::ZERO),
                        r: node.p70_vec("Lcl Rotation", DVec3::ZERO),
                        turned: None,
                        s: node.p70_vec("Lcl Scaling", DVec3::ONE),
                    };
                    for (k, layer) in layers.iter().enumerate() {
                        let Some(channels) = layer.driven.get(&id) else {
                            continue;
                        };
                        for (slot, ch) in channels.iter().enumerate().filter(|(_, ch)| ch.on) {
                            let mut v = DVec3::from_array(ch.rest);
                            for axis in 0..3 {
                                if let Some(c) = &ch.curves[axis] {
                                    v[axis] = c.at(time);
                                }
                            }
                            at.lay(k == 0, layer, time, slot, v, order);
                        }
                    }
                    let turn = at.turned.unwrap_or_else(|| euler(at.r, order));
                    local_turned(node, at.t, &turn, at.s)
                };
                let mut pose = Vec::with_capacity(space.ids.len());
                for (b, &id) in space.ids.iter().enumerate() {
                    let mut m = Affine::IDENTITY;
                    let mut at = id;
                    for _ in 0..256 {
                        let Some(node) = by_id.get(&at) else { break };
                        m = local_at(at, node).then(&m);
                        match parent_of.get(&at) {
                            Some(&p) if p != 0 => at = p,
                            _ => break,
                        }
                    }
                    let k = space
                        .out
                        .then(&conj(&m))
                        .then(&space.unlink[b])
                        .then(&space.mesh[b]);
                    pose.push([
                        k.x.x, k.x.y, k.x.z, 0.0, k.y.x, k.y.y, k.y.z, 0.0, k.z.x, k.z.y, k.z.z,
                        0.0, k.t.x, k.t.y, k.t.z, 1.0,
                    ]);
                }
                poses.push(pose);
            }
            // Each channel's weight at each time, through the layers.
            let mut morphs = Vec::new();
            let driven = |c: &(i64, Vec<f64>)| layers.iter().any(|l| l.shapes.contains_key(&c.0));
            if channels.iter().any(driven) {
                for &time in &times {
                    let mut row = Vec::new();
                    for (c, keys) in channels {
                        let mut at = Layered {
                            t: DVec3::new(by_id.get(c).map_or(0.0, deform_percent), 0.0, 0.0),
                            r: DVec3::ZERO,
                            turned: None,
                            s: DVec3::ONE,
                        };
                        for (k, layer) in layers.iter().enumerate() {
                            if let Some((curve, rest)) = layer.shapes.get(c) {
                                let v = curve.as_ref().map_or(*rest, |x| x.at(time));
                                at.lay(k == 0, layer, time, 0, DVec3::new(v, 0.0, 0.0), 0);
                            }
                        }
                        row.extend(shape_weights(at.t.x / 100.0, keys));
                    }
                    morphs.push(row);
                }
            }
            clips.push(crate::mesh::SkinClip {
                morphs,
                name: unique(sname, &mut used),
                times: times.iter().map(|t| t - start).collect(),
                poses,
            });
        }
        Ok(clips)
    };
    // A video's name and its bytes (embedded content, or the file).
    let video = |v: &Node| -> (String, Result<Vec<u8>, String>) {
        let file = v
            .text("RelativeFilename")
            .or(v.text("Filename"))
            .unwrap_or("")
            .replace('\\', "/");
        let name = if file.is_empty() {
            object_name(v.props.get(1).and_then(Prop::str).unwrap_or("video"))
        } else {
            file.clone()
        };
        let content = v.child("Content").and_then(|c| match c.props.first() {
            Some(Prop::Bytes(b)) if !b.is_empty() => Some(b.clone()),
            _ => None,
        });
        let bytes = match content {
            Some(b) => Ok(b),
            None => fetch(&file, resolve).map_err(|e| e.to_string()),
        };
        (name, bytes)
    };
    for o in objects.all("Model") {
        let Some(id) = o.props.first().and_then(Prop::int) else {
            continue;
        };
        let geometries: Vec<&Node> = children_of
            .get(&id)
            .map(|cs| {
                cs.iter()
                    .filter_map(|c| by_id.get(c))
                    .filter(|n| {
                        n.name == "Geometry" && n.props.get(2).and_then(Prop::str) == Some("Mesh")
                    })
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        let materials: Vec<(i64, String)> = children_of
            .get(&id)
            .map(|cs| {
                cs.iter()
                    .filter_map(|c| Some((*c, *by_id.get(c)?)))
                    .filter(|(_, n)| n.name == "Material")
                    .map(|(c, n)| {
                        (
                            c,
                            object_name(n.props.get(1).and_then(Prop::str).unwrap_or("")),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        // The first material, and its textures through their videos.
        let mut unread: Vec<String> = Vec::new();
        let material = materials.first().map(|(m, mname)| {
            let node = by_id.get(m);
            let texture = |prop: &str| {
                let t = textures_of.get(m)?.get(prop)?;
                let v = children_of
                    .get(t)?
                    .iter()
                    .filter_map(|c| by_id.get(c))
                    .find(|n| n.name == "Video")?;
                let (name, bytes) = video(v);
                Some(Texture::from_bytes(&name, bytes.ok()?))
            };
            if let Some(links) = textures_of.get(m) {
                for prop in links.keys() {
                    if !["DiffuseColor", "NormalMap", "EmissiveColor"].contains(&prop.as_str()) {
                        unread.push(prop.clone());
                    }
                }
            }
            let rgb = |key: &str, factor: &str| {
                let n = node?;
                let c = n.p70_vec(key, DVec3::NAN);
                if !c.is_finite() {
                    return None;
                }
                let f = n.p70_num(factor).unwrap_or(1.0);
                Some([c.x * f, c.y * f, c.z * f])
            };
            Material {
                name: mname.clone(),
                color: rgb("DiffuseColor", "DiffuseFactor"),
                emissive: rgb("EmissiveColor", "EmissiveFactor").unwrap_or([0.0; 3]),
                opacity: node
                    .and_then(|n| n.p70_num("Opacity"))
                    .filter(|x| x.is_finite()),
                map: texture("DiffuseColor"),
                normal_map: texture("NormalMap"),
                emissive_map: texture("EmissiveColor"),
                ..Default::default()
            }
        });
        let name = unique(
            &object_name(o.props.get(1).and_then(Prop::str).unwrap_or("")),
            &mut used,
        );
        let geo = geometric(o);
        for g in geometries {
            let (mut solid, uvs, colors, polys) =
                mesh(g, &name, &geo, unit, &to_kernel, p_det < 0.0)?;
            if colors.is_none() && g.all("LayerElementColor").next().is_some() {
                out.skipped.push(format!(
                    "{name}: its colour layer misses some corners, so no vertex colours"
                ));
            }
            solid.colors = colors;
            if uvs.is_none() && g.all("LayerElementUV").next().is_some() {
                out.skipped.push(format!(
                    "{name}: its UV layer misses some corners, so no UVs"
                ));
            }
            solid.uvs = uvs;
            // Blend shapes: each channel a morph target, its last shape's
            // offsets (control points the shape does not list stay).
            let (mut targets, channels) = shapes_of(
                g,
                &|d: DVec3| to_kernel(geo.rotate(d)) * unit,
                solid.positions.len(),
            );
            let read = skin_of(g, &name, &solid.positions)?;
            let mut clips = Vec::new();
            if let Some(((skin, scaled), space)) = read {
                clips = clips_of(&space, &channels)?;
                if let Some((placed, info)) = scaled {
                    // Where the file's bind pose puts the vertices, and
                    // each target's ends with them.
                    for t in &mut targets {
                        let moved: Vec<DVec3> = solid
                            .positions
                            .iter()
                            .zip(&t.deltas)
                            .map(|(p, d)| *p + *d)
                            .collect();
                        if let Some(((_, Some((ends, _))), _)) = skin_of(g, &name, &moved)? {
                            for ((d, e), p) in t.deltas.iter_mut().zip(&ends).zip(&placed) {
                                *d = *e - *p;
                            }
                        }
                    }
                    solid.positions = placed;
                    out.skin_scales.push(info);
                }
                solid.skin = Some(skin);
            } else if !channels.is_empty() {
                // No bones: the clips drive the targets alone.
                let none = FbxSpace {
                    ids: Vec::new(),
                    unlink: Vec::new(),
                    mesh: Vec::new(),
                    out: Affine::IDENTITY,
                };
                clips = clips_of(&none, &channels)?;
            }
            if !targets.is_empty() {
                let mut names = BTreeMap::new();
                solid.morphs = Some(crate::mesh::Morphs {
                    targets: targets
                        .into_iter()
                        .map(|mut t| {
                            t.name = unique(&t.name, &mut names);
                            t
                        })
                        .collect(),
                });
            }
            let mut transform = conj(&world(id));
            // A mesh with no skin that an animation moves by its model
            // (or a model above it) is carried as a skin of one bone, as
            // a glTF mesh moved by its node is (`docs/research.md` §47).
            let driven = {
                let mut at = id;
                let mut found = false;
                for _ in 0..256 {
                    if stacks
                        .iter()
                        .any(|(_, layers)| layers.iter().any(|l| l.driven.contains_key(&at)))
                    {
                        found = true;
                        break;
                    }
                    match parent_of.get(&at) {
                        Some(&p) if p != 0 => at = p,
                        _ => break,
                    }
                }
                found
            };
            let rest = crate::tools::import::affine_inverse(&transform)
                .filter(|_| driven && solid.skin.is_none() && transform.determinant() > 0.0);
            if let Some(unlink) = rest {
                let space = FbxSpace {
                    ids: vec![id],
                    unlink: vec![unlink],
                    mesh: vec![Affine::IDENTITY],
                    out: Affine::IDENTITY,
                };
                let moved = clips_of(&space, &channels)?;
                let placed = crate::tools::repair::apply_transform(&ImportedPart {
                    name: name.clone(),
                    solid: solid.clone(),
                    transform,
                    ..Default::default()
                });
                if let (false, Ok(mut placed)) = (moved.is_empty(), placed) {
                    placed.skin = Some(crate::mesh::Skin {
                        bones: vec![crate::mesh::Bone::new(
                            &format!("{name}.node"),
                            None,
                            crate::math::Frame::IDENTITY,
                        )],
                        weights: vec![crate::mesh::Weights::single(0); placed.positions.len()],
                    });
                    solid = placed;
                    clips = moved;
                    transform = Affine::IDENTITY;
                    out.skipped.push(format!(
                        "{name}: moved by its model's animation, so carried as a skin of one bone ({name}.node)"
                    ));
                }
            } else if solid.skin.is_some() {
                // The bones place a skinned mesh; its vertices and bones
                // share the mesh's own coordinates.
                if geo != Affine::IDENTITY || p_det < 0.0 {
                    return Err(ImportError::Unsupported(format!(
                        "{name}: a skinned mesh with a geometric transform or mirrored axes"
                    )));
                }
                if transform != Affine::IDENTITY {
                    out.skipped.push(format!(
                        "{name}: skinned, so its model transform is not used (its bones place it)"
                    ));
                }
                transform = Affine::IDENTITY;
            }
            out.polygons_triangulated += polys;
            out.parts.push(ImportedPart {
                clips,
                name: name.clone(),
                solid,
                transform,
                material: material.clone().unwrap_or_default(),
                defaults: Vec::new(),
            });
            if !unread.is_empty() {
                out.skipped.push(format!(
                    "{name}: its material's textures on {} are not read",
                    unread.join(", ")
                ));
            }
        }
    }
    // Texture images.
    let mut seen = BTreeMap::new();
    for v in objects.all("Video") {
        let (name, bytes) = video(v);
        if seen.insert(name.clone(), ()).is_some() {
            continue;
        }
        out.images.push(match bytes {
            Ok(b) => ImageInfo::from_bytes(&name, &b),
            Err(e) => ImageInfo::missing(&name, &e),
        });
    }
    if out.parts.is_empty() {
        let what = if out.skipped.is_empty() {
            String::new()
        } else {
            format!(" (it has {})", out.skipped.join("; "))
        };
        return Err(ImportError::Unsupported(format!(
            "an FBX file with no polygon meshes{what}"
        )));
    }
    Ok(out)
}

use crate::tools::import::ReadSkin as FbxSkin;

#[cfg(test)]
mod tests;
